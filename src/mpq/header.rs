use std::{fs, mem};
use std::io::{SeekFrom, Seek, Read};

use bitflags::bitflags;

use byteorder::{ByteOrder, LittleEndian};

use anyhow::{bail, Context};

use super::crypto;

/// Magic number for the MPQ A file marker
const HEADER_MAGIC: &[u8] = b"MPQ\x1A";
/// The header size will always be the same
const HEADER_SIZE: usize = 32;
/// Diablo 1 uses version 0
const HEADER_VERSION: u16 = 0;

/// Denotes a type that can be read from a byte array
pub trait ByteReadable {
    /// Initialize an instance from a byte array
    fn read(bytes: &[u8]) -> Self;

    /// Read and decrypt many instances from a file
    /// Offset is the byte offset into the file
    /// Count is the number of instances to read
    /// Seed is the decryption key
    fn read_and_decrypt_many<T: ByteReadable>(
        file: &mut fs::File,
        offset: usize,
        count: usize,
        seed: u32,
    ) -> anyhow::Result<Vec<T>> {
        // Allocate a buffer large enough to hold all entries
        let size = count * mem::size_of::<T>();
        let mut buffer = vec![0x0u8; size];
        // Seek and read from the file handle
        file.seek(SeekFrom::Start(offset as u64))
            .context("Failed to seek")?;
        file.read_exact(&mut buffer)
            .context("Failed to read from file")?;
        // Decrypt the buffer contents
        crypto::decrypt(&mut buffer, seed);
        // Map the buffer contents into a vector
        let mut entries: Vec<T> = Vec::with_capacity(count);
        for i in 0..count {
            // Calculate the start of this entry
            let start = i as usize * mem::size_of::<T>();
            // Read the buffer contents into a new instance, push to the entries vector
            entries.push(T::read(&buffer[start..]));
        }
        Ok(entries)
    }
}

/// MPQ file header
#[derive(Debug)]
pub struct Header {
    pub magic: [u8; 4],
    pub header_size: u32,
    pub file_size: u32,
    pub version: u16,
    pub block_size_factor: u16,

    pub hash_table_offset: u32,
    pub block_table_offset: u32,
    pub hash_table_count: u32,
    pub block_table_count: u32,
    // __padding: [u8; 72],
}

impl Header {
    /// Try to find the MPQ header and it's offset in the file
    pub fn find_in_file(file: &mut fs::File) -> anyhow::Result<(Self, usize)> {
        // Get the size of the file
        let archive_size = file.metadata()
            .context("Failed to get file metadata")?
            .len() as usize;
        // Offset into the archive that the header was found at
        let mut archive_offset = 0usize;
        // Mutable buffer to read the header data into
        let mut buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        loop {
            // Seek to the next archive offset and read
            file.seek(SeekFrom::Start(archive_offset as u64))
                .context("Failed to seek into file")?;
            file.read_exact(&mut buffer)
                .context("Failed to read from file")?;
            // Read and validate the header
            let header = Header::read(&buffer);
            if header.is_valid() {
                return Ok((header, archive_offset));
            }
            // Valid header not found, continue to the next sector 
            archive_offset += 512;
            // If the archive offset has exceeded the file size, bail
            if archive_offset > archive_size {
                bail!("Failed to find valid header in file");
            }
        }
    }
    /// Check if the MPQ header is valid
    pub fn is_valid(&self) -> bool {
        (self.magic == HEADER_MAGIC) && (self.header_size == HEADER_SIZE as u32) && (self.version == HEADER_VERSION)
    }
}

impl ByteReadable for Header {
    fn read(bytes: &[u8]) -> Self {
        Self {
            magic: [bytes[0], bytes[1], bytes[2], bytes[3]],
            header_size: LittleEndian::read_u32(&bytes[0x04..]),
            file_size: LittleEndian::read_u32(&bytes[0x08..]),
            version: LittleEndian::read_u16(&bytes[0x0C..]),
            block_size_factor: LittleEndian::read_u16(&bytes[0x0E..]),
            hash_table_offset: LittleEndian::read_u32(&bytes[0x10..]),
            block_table_offset: LittleEndian::read_u32(&bytes[0x14..]),
            hash_table_count: LittleEndian::read_u32(&bytes[0x18..]),
            block_table_count: LittleEndian::read_u32(&bytes[0x1C..]),
        }
    }
}

bitflags! {
    /// Bit flags for file block entries
    pub struct BlockFlags : u32 {
        /// Marker that this file exists
        const EXISTS    = 0x80000000;
        /// Marker that this file is encrypted
        const ENCRYPTED = 0x00010000;
        /// Marker that this file uses PkWare data compression
        const COMPRESS_PKWARE = 0x00000100;
        /// Marker that this file uses multiple compressions
        const COMPRESS_MULTI = 0x00000200;
        /// Single sector file storage
        /// Probably not used in Diablo, first appeared in WOW
        const SINGLE = 0x01000000;

        const ANY_COMPRESSION = Self::COMPRESS_PKWARE.bits | Self::COMPRESS_MULTI.bits;
    }
}

/// Denotes that this hash entry is free (never used)
pub const BLOCK_INDEX_FREE: u32 = 0xFFFFFFFF;
/// Denotes that this hash entry has been deleted
pub const BLOCK_INDEX_DELETED: u32 = 0xFFFFFFFF;

/// Entry in the MPQ archive hash table
#[derive(Debug)]
pub struct HashEntry {
    // Filename hashes
    pub hash_a: u32,
    pub hash_b: u32,
    // Locale marker, maybe windows related?
    pub locale: u16,
    // Platform marker, seems to always be 0
    pub platform: u16,
    // Index into the block table for this file
    // The BLOCK_INDEX_* values are special markers
    pub block_index: u32,
}

impl ByteReadable for HashEntry {
    fn read(bytes: &[u8]) -> Self {
        Self {
            hash_a: LittleEndian::read_u32(bytes),
            hash_b: LittleEndian::read_u32(&bytes[4..]),
            locale: LittleEndian::read_u16(&bytes[8..]),
            platform: LittleEndian::read_u16(&bytes[10..]),
            block_index: LittleEndian::read_u32(&bytes[12..]),
        }
    }
}

/// Entry in the MPQ archive block table
#[derive(Debug, Copy, Clone)]
pub struct BlockEntry {
    // Byte offset of the file from the start of the archive
    pub offset: u32,
    // Compressed size of the file, in bytes
    pub size_packed: u32,
    // Uncompressed size of the file, in bytes
    pub size_unpacked: u32,
    // Flags
    pub flags: BlockFlags,
}

impl ByteReadable for BlockEntry {
    fn read(bytes: &[u8]) -> Self {
        Self {
            offset: LittleEndian::read_u32(bytes),
            size_packed: LittleEndian::read_u32(&bytes[4..]),
            size_unpacked: LittleEndian::read_u32(&bytes[8..]),
            flags: BlockFlags{ bits: LittleEndian::read_u32(&bytes[12..]) },
        }
    }
}

impl BlockEntry {
    pub fn exists(&self) -> bool {
        !(self.flags & BlockFlags::EXISTS).is_empty()
    }

    pub fn is_compressed(&self) -> bool {
        !(self.flags & BlockFlags::ANY_COMPRESSION).is_empty()
    }

    pub fn is_imploded(&self) -> bool {
        !(self.flags & BlockFlags::COMPRESS_PKWARE).is_empty()
    }

    pub fn is_encrypted(&self) -> bool {
        !(self.flags & BlockFlags::ENCRYPTED).is_empty()
    }

    pub fn has_muli_compression(&self) -> bool {
        !(self.flags & BlockFlags::COMPRESS_MULTI).is_empty()
    }
}

/// Iterator for the sectors that contain a file
/// This iterator returns (offset, size) tuples
#[derive(Debug)]
pub struct FileSectors {
    index: usize,
    offsets: Vec<u32>,
}

impl FileSectors {
    pub fn get(
        file: &mut fs::File, 
        file_key: Option<u32>,
        offset: usize,
        size_unpacked: usize,
        sector_size: usize,
    ) -> anyhow::Result<Self> {
        // Get the number of sectors this file takes up
        let sector_count = ((size_unpacked - 1) as usize / sector_size) + 1;
        // Read the sector offsets into a buffer
        let mut buffer = vec![0x0u8; (sector_count + 1) * 4];
        file.seek(SeekFrom::Start(offset as u64))
            .context("Failed to seek to file sector")?;
        file.read_exact(&mut buffer)
            .context("Failed to read sectors")?;
        // If the block is encrypted, decrypt
        if let Some(key) = file_key {
            crypto::decrypt(&mut buffer, key - 1);
        }
        // Map the sectors into u32s
        let mut offsets: Vec<u32> = Vec::with_capacity(sector_count+1);
        for i in 0..=sector_count {
            offsets.push(LittleEndian::read_u32(&buffer[i*4..]));
        }

        Ok(Self {
            index: 0,
            offsets,
        })
    }
}

impl Iterator for FileSectors {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < (self.offsets.len() - 1) {
            let offset = self.offsets[self.index] as usize;
            let size = self.offsets[self.index + 1] as usize - offset;

            self.index += 1;

            Some((offset, size))
        } else {
            None
        }
    }
}
