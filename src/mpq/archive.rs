use std::fs;
use std::path::Path;
use std::io::{Read, Seek, SeekFrom};
use std::io::{Error, ErrorKind, Result};

use super::header::*;
use super::{crypto, compression};
use super::crypto::HashType;

/// MPQ data archive
/// This is *not* intended as a complete implementation of the MPQ file format, just one usable enough for this project
/// NOTE: Big thanks to the libmpq library by ge0rg
/// https://github.com/ge0rg/libmpq/blob/master/libmpq/mpq-internal.h
#[derive(Debug)]
pub struct Archive {
    // Archive file handle
    file: fs::File,
    // Byte offset into the file at which the archive was found
    offset: usize,
    // Size of the sector blocks used to store files
    // Calculated as 512 << header.block_size_factor
    sector_size: usize,
    // Archive file header
    header: Header,
    // Lookup tables for files
    hash_table: Vec<HashEntry>,
    block_table: Vec<BlockEntry>,
}

impl Archive {
    /// Open an existing archive from the file system
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Hash the seeds for the archive tables
        let hash_table_seed = crypto::hash("(hash table)", HashType::FileKey);
        let block_table_seed = crypto::hash("(block table)", HashType::FileKey);
        // Open the file from the path
        let mut file = fs::File::open(path)?;
        // Read and validate the header
        // If the header is not present (or is invalid), there's no need to proceed
        let (header, offset) = Header::find_in_file(&mut file)?;
        // Calculate the size of block sectors
        let sector_size = 512 << header.block_size_factor;
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
            offset,
            sector_size,
            header,
            hash_table,
            block_table,
        })
    }

    /// Check if a file exists in the archive
    pub fn has_file(&self, filename: &str) -> bool {
        self.get_block_index(filename).is_some()
    }

    /// Get the handle for a file in the archive
    pub fn get_file(&self, filename: &str) -> Result<File> {
        // Get the block
        let block = self.get_block_index(filename)
            .and_then(|index| Some(&self.block_table[index as usize]))
            .ok_or(Error::new(ErrorKind::NotFound, "Failed to get block for file"))?;
        if !block.exists() {
            return Err(Error::new(ErrorKind::NotFound, "File block marked as non-existant"))?;
        }
        // If the file is encrypted, get the encryption key
        let file_key = if block.is_encrypted() {
            // Get the last portion of the filename, without the path info
            let filename = filename.split(&['\\', '/'][..])
                .last()
                .ok_or(Error::new(ErrorKind::InvalidData, "Failed to extract filename from path"))?;
            // Hash the filename as the encryption key
            Some(crypto::hash(filename, HashType::FileKey))
        } else {
            None
        };

        Ok(File {
            key: file_key,
            block: *block,
            archive: self,
        })
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

/// A handle pointing to a file stored in a MPQ Archive
#[derive(Debug, Clone, Copy)]
pub struct File<'a> {
    key: Option<u32>,
    block: BlockEntry,
    archive: &'a Archive,
}

impl<'a> File<'a> {
    /// Get the decompressed file size
    pub fn size(&self) -> usize {
        self.block.size_unpacked as usize
    }

    /// Read the file into the out buffer. The out buffer must be large enough to hold the complete file.
    /// NOTE: This matches the std::io::Read, but cannot be implemented directly.
    /// This is because the whole file must be read at once, due to the way compression and encryption work.
    pub fn read(&self, out: &mut [u8]) -> Result<usize> {
        // Check that the file can be read into the supplied output buffer
        if out.len() < self.block.size_unpacked as usize {
            return Err(Error::new(ErrorKind::InvalidInput, "Output buffer not large enough for unpacked file"));
        }
        // Clone the file handle to keep the archive as immutable
        let mut file = self.archive.file.try_clone()?;
        // Get the block and file offset
        let block = &self.block;
        let offset = self.archive.offset + block.offset as usize;
        // If the file is not compressed
        if !block.is_compressed() {
            // Just read the file directly
            file.seek(SeekFrom::Start(offset as u64))?;
            file.read_exact(out)?;
            Ok(self.block.size_unpacked as usize)
        } else {
            // Allocate a sector-sized buffer to read into
            let mut buffer = vec![0x0u8; self.archive.sector_size];
            // Keep track of the number of bytes written
            let mut bytes_written = 0usize;
            // Get the sectors that this file is stored in
            let sectors = FileSectors::get(
                &mut file, self.key, 
                offset, self.block.size_unpacked as usize, 
                self.archive.sector_size
            )?;
            for (index, sector) in sectors.enumerate() {
                // Decompose the sector into it's offset and size
                let (sector_offset, sector_size) = sector;
                // Get the input and output buffers
                let output: &mut [u8] = &mut out[bytes_written..];
                let mut input: &mut [u8] = &mut buffer[0..sector_size];
                // Get the offset into the archive for this sector
                let offset = self.archive.offset + (block.offset as usize) + sector_offset;
                // Read the sector into the input buffer
                file.seek(SeekFrom::Start(offset as u64))?;
                file.read_exact(input)?;
                // If the sector is encrypted, decrypt it
                if let Some(key) = self.key {
                    crypto::decrypt(&mut input, key + index as u32);
                }
                // Apply decompression 
                bytes_written += if block.is_imploded() {
                    compression::explode_into(input, output)?
                } else if block.has_muli_compression() {
                    compression::decompress_into(input, output)?
                } else {
                    todo!()
                };
            }
            Ok(bytes_written)
        }
    }
}
