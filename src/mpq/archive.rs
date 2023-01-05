use std::mem;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Seek, SeekFrom};

use bitflags::bitflags;

use byteorder::{ByteOrder, LittleEndian};

use anyhow::{bail, Context};

use super::crypto;
use super::crypto::HashType;

/// NOTE: Big thanks to the libmpq library by ge0rg
/// https://github.com/ge0rg/libmpq/blob/master/libmpq/mpq-internal.h

/// Magic number for the MPQ A file marker
const HEADER_MAGIC: &[u8] = b"MPQ\x1A";
/// The header size will always be the same
const HEADER_SIZE: usize = 32;
/// Diablo 1 uses version 0
const HEADER_VERSION: u16 = 0;

bitflags! {
    /// Bit flags for file block entries
    struct BlockFlags : u32 {
        /// Marker that this file exists
        const EXISTS    = 0x80000000;
        /// Marker that this file is encrypted
        const ENCRYPTED = 0x00010000;
        /// Marker that this file uses PkWare data compression
        const COMPRESS_IMPLODE = 0x00000100;
        /// Marker that this file uses multiple compressions
        const COMPRESS_MULTI = 0x00000200;
        /// Single sector file storage
        /// Probably not used in Diablo, first appeared in WOW
        const SINGLE = 0x01000000;
    }
}

/// MPQ data archive
/// This is the data archive format used by Blizzard, first starting in Diablo
/// It utilizes a linearly-probed hash table for indirect file lookup in the archive itself
#[derive(Debug)]
pub struct Archive {
    file: File,

    block_size: usize,
    archive_offset: usize,

    header: Header,
    hash_table: Vec<HashEntry>,
    block_table: Vec<BlockEntry>,
}

impl Archive {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // Hash the seeds for the archive tables
        let hash_table_seed = crypto::hash("(hash table)", HashType::FileKey);
        let block_table_seed = crypto::hash("(block table)", HashType::FileKey);
        // Open the file from the path
        let mut file = File::open(path).context("Failed to open file")?;
        // Read and validate the header
        // If the header is not present (or is invalid), there's no need to proceed
        let (header, archive_offset) = Header::find_in_file(&mut file)?;
        // Calculate the block size
        let block_size = 512 << header.block_size_factor;
        // Read and decrypt the hash table
        let hash_table = HashEntry::read_and_decrypt_many(
            &mut file,
            header.hash_table_offset as usize,
            header.hash_table_count as usize,
            hash_table_seed,
        )?;
        // Read and decrypt the file block table
        let block_table = BlockEntry::read_and_decrypt_many(
            &mut file,
            header.block_table_offset as usize,
            header.block_table_count as usize,
            block_table_seed,
        )?;

        Ok(Self {
            file,
            archive_offset,
            block_size,
            header,
            hash_table,
            block_table,
        })
    }

    pub fn has_file(&self, filename: &str) -> bool {
        self.get_block_index(filename).is_some()
    }

    fn get_block_index(&self, filename: &str) -> Option<usize> {
        // Hash the filename into a couple of different markers
        let hash_i = crypto::hash(filename, HashType::TableOffset);
        let hash_a = crypto::hash(filename, HashType::NameA);
        let hash_b = crypto::hash(filename, HashType::NameB);
        // Get the length of the hashtable
        let len = self.header.hash_table_count - 1;
        // Get the starting index based on the hash function
        let start_index = hash_i & len;
        // Linear probing
        let mut i = start_index;
        loop {
            // If the hashes match, return the table index
            let hash = &self.hash_table[i as usize];
            if hash.hash_a == hash_a && hash.hash_b == hash_b {
                let block = &self.block_table[hash.block_index as usize];
                println!("{:?}", block);
                return Some(hash.block_index as usize);
            }
            // No match found, iterate
            i = (i + 1) & len;
            // If this block index was marked as 'never used',
            // or we're back at the starting index, return None
            if hash.block_index == BLOCK_INDEX_FREE || i == start_index {
                return None;
            }
        }
    }
}

/// Denotes a type that can be read from a byte array
trait ByteReadable {
    /// Initialize an instance from a byte array
    fn read(bytes: &[u8]) -> Self;

    /// Read and decrypt many instances from a file
    /// Offset is the byte offset into the file
    /// Count is the number of instances to read
    /// Seed is the decryption key
    fn read_and_decrypt_many<T: ByteReadable>(
        file: &mut File,
        offset: usize,
        count: usize,
        seed: u32,
    ) -> anyhow::Result<Vec<T>> {
        let size = count * mem::size_of::<T>();
        let mut buffer = vec![0x0u8; size];

        file.seek(SeekFrom::Start(offset as u64))
            .context("Failed to seek")?;
        file.read_exact(&mut buffer)
            .context("Failed to read from file")?;

        crypto::decrypt(&mut buffer, seed);

        let mut entries: Vec<T> = Vec::with_capacity(count);
        for i in 0..count {
            let start = i as usize * mem::size_of::<T>();
            entries.push(T::read(&buffer[start..]));
        }
        Ok(entries)
    }
}

/// MPQ file header
#[derive(Debug)]
struct Header {
    magic: [u8; 4],
    header_size: u32,
    _file_size: u32,
    version: u16,
    block_size_factor: u16,

    hash_table_offset: u32,
    block_table_offset: u32,
    hash_table_count: u32,
    block_table_count: u32,
    // __padding: [u8; 72],
}

impl Header {
    /// Try to find the MPQ header and it's offset in the file
    fn find_in_file(file: &mut File) -> anyhow::Result<(Self, usize)> {
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
    fn is_valid(&self) -> bool {
        (self.magic == HEADER_MAGIC) && (self.header_size == HEADER_SIZE as u32) && (self.version == HEADER_VERSION)
    }
}

impl ByteReadable for Header {
    fn read(bytes: &[u8]) -> Self {
        Self {
            magic: [bytes[0], bytes[1], bytes[2], bytes[3]],
            header_size: LittleEndian::read_u32(&bytes[0x04..]),
            _file_size: LittleEndian::read_u32(&bytes[0x08..]),
            version: LittleEndian::read_u16(&bytes[0x0C..]),
            block_size_factor: LittleEndian::read_u16(&bytes[0x0E..]),
            hash_table_offset: LittleEndian::read_u32(&bytes[0x10..]),
            block_table_offset: LittleEndian::read_u32(&bytes[0x14..]),
            hash_table_count: LittleEndian::read_u32(&bytes[0x18..]),
            block_table_count: LittleEndian::read_u32(&bytes[0x1C..]),
        }
    }
}

/// Denotes that this hash entry is free (never used)
const BLOCK_INDEX_FREE: u32 = 0xFFFFFFFF;
/// Denotes that this hash entry has been deleted
const _BLOCK_INDEX_DELETED: u32 = 0xFFFFFFFF;

/// Entry in the MPQ archive hash table
#[derive(Debug)]
struct HashEntry {

    // Filename hashes
    hash_a: u32,
    hash_b: u32,
    // Locale marker, maybe windows related?
    _locale: u16,
    // Platform marker, seems to always be 0
    _platform: u16,
    // Index into the block table for this file
    // The BLOCK_INDEX_* values are special markers
    block_index: u32,
}

impl ByteReadable for HashEntry {
    fn read(bytes: &[u8]) -> Self {
        Self {
            hash_a: LittleEndian::read_u32(bytes),
            hash_b: LittleEndian::read_u32(&bytes[4..]),
            _locale: LittleEndian::read_u16(&bytes[8..]),
            _platform: LittleEndian::read_u16(&bytes[10..]),
            block_index: LittleEndian::read_u32(&bytes[12..]),
        }
    }
}

/// Entry in the MPQ archive block table
#[derive(Debug, Copy, Clone)]
struct BlockEntry {
    // Byte offset of the file from the start of the archive
    offset: u32,
    // Compressed size of the file, in bytes
    size_packed: u32,
    // Uncompressed size of the file, in bytes
    size_unpacked: u32,
    // Flags
    flags: BlockFlags,
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
