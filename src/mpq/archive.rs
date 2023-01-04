use std::fs::File;
use std::mem;
use std::path::Path;

use std::io::Read;
use std::io::{Seek, SeekFrom};

use byteorder::{ByteOrder, LittleEndian};

use anyhow::{bail, Context};

use super::crypto;
use super::crypto::HashType;

const HEADER_MAGIC: &[u8] = b"MPQ\x1A";
const HEADER_SIZE: usize = 32;
const HEADER_VERSION: u16 = 0;

#[derive(Debug)]
pub struct Archive {
    file: File,
    header: Header,
    hash_table: Vec<HashEntry>,
    block_table: Vec<BlockEntry>,
}

impl Archive {
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let hash_table_seed = crypto::hash("(hash table)", HashType::FileKey);
        let block_table_seed = crypto::hash("(block table)", HashType::FileKey);

        let mut file = File::open(path).context("Failed to open file")?;

        let header = {
            let mut buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
            file.seek(SeekFrom::Start(0)).context("Failed to seek")?;
            file.read_exact(&mut buffer)
                .context("Failed to read from file")?;

            let h = Header::read(&buffer);
            h.validate()?;
            h
        };

        let hash_table = HashEntry::read_and_decrypt_many(
            &mut file,
            header.hash_table_offset as usize,
            header.hash_table_count as usize,
            hash_table_seed,
        )?;

        let block_table = BlockEntry::read_and_decrypt_many(
            &mut file,
            header.block_table_offset as usize,
            header.block_table_count as usize,
            block_table_seed,
        )?;

        Ok(Self {
            file,
            header,
            hash_table,
            block_table,
        })
    }

    pub fn has_file(&self, filename: &str) -> bool {
        self.get_block_index(filename).is_some()
    }

    pub fn file_size(&self, filename: &str) -> Option<(u32, u32)> {
        self.get_block_index(filename)
            .and_then(|index| Some(self.block_table[index as usize]))
            .and_then(|block| Some((block.size_packed, block.size_unpacked)))
    }

    fn get_block_index(&self, filename: &str) -> Option<u32> {
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
                return Some(hash.block_index);
            }
            // No match found, iterate
            i = (i + 1) & len;
            // If this block index was marked as 'never used',
            // or we're back at the starting index, return None
            if hash.block_index == 0xFFFFFFFF || i == start_index {
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
    file_size: u32,
    version: u16,
    block_size_factor: u16,

    hash_table_offset: u32,
    block_table_offset: u32,
    hash_table_count: u32,
    block_table_count: u32,
    // __padding: [u8; 72],
}

impl Header {
    fn validate(&self) -> anyhow::Result<()> {
        if self.magic != HEADER_MAGIC {
            bail!("MPQ File signature does not match {:?}", HEADER_MAGIC);
        }
        if self.header_size != HEADER_SIZE as u32 {
            bail!("MPQ File header size does not match {:?}", HEADER_SIZE);
        }
        if self.version != HEADER_VERSION {
            bail!(
                "MPQ File header version does not match {:?}",
                HEADER_VERSION
            );
        }
        Ok(())
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

/// Entry in the MPQ archive hash table
#[derive(Debug)]
struct HashEntry {
    // Filename hashes
    hash_a: u32,
    hash_b: u32,
    // Locale marker, maybe windows related?
    locale: u16,
    // Platform marker, seems to always be 0
    platform: u16,
    // Index into the block table for this file
    // 	0xFFFFFFFF -> File has never been used
    // 	0xFFFFFFFE -> File has been deleted
    block_index: u32,
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
struct BlockEntry {
    // Byte offset of the file from the start of the archive
    offset: u32,
    // Compressed size of the file, in bytes
    size_packed: u32,
    // Uncompressed size of the file, in bytes
    size_unpacked: u32,
    // Flags
    flags: u32,
}

impl ByteReadable for BlockEntry {
    fn read(bytes: &[u8]) -> Self {
        Self {
            offset: LittleEndian::read_u32(bytes),
            size_packed: LittleEndian::read_u32(&bytes[4..]),
            size_unpacked: LittleEndian::read_u32(&bytes[8..]),
            flags: LittleEndian::read_u32(&bytes[12..]),
        }
    }
}
