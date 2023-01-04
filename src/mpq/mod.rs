use std::mem;
use std::fs::File;
use std::path::Path;

use std::io::{BufReader, Read};
use std::io::{Seek, SeekFrom};

use anyhow::{Context, bail};

const MPQ_HEADER_SIGNATURE: u32 = u32::from_be_bytes(['\x1A' as u8, 'Q' as u8, 'P' as u8, 'M' as u8]);
const MPQ_HEADER_SIZE: u32 = 32;
const MPQ_HEADER_VERSION: u16 = 0;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct FileHeader {
	signature: u32,
	size: u32,
	file_size: u32,
	version: u16,
	block_size_factor: u16,
	hash_entries_offset: u32,
	block_entries_offset: u32,
	hash_entries_count: u32,
	block_entries_count: u32,
	// __padding: [u8; 72],
}

impl FileHeader {
	unsafe fn read_from_buf<R: Read>(reader: &mut BufReader<R>) -> anyhow::Result<Self> {
		let mut header: Self = mem::zeroed();

		let size = mem::size_of::<Self>();
		let slice = std::slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, size);
		reader
			.read_exact(slice)
			.context("Failed to read file header")?;
		header.validate()?;

		Ok(header)
	}

	fn validate(&self) -> anyhow::Result<()> {
		if self.signature != MPQ_HEADER_SIGNATURE {
			bail!("MPQ File signature does not match {:?}", MPQ_HEADER_SIGNATURE);
		}
		if self.size != MPQ_HEADER_SIZE {
			bail!("MPQ File header size does not match {:?}", MPQ_HEADER_SIZE);
		}
		if self.version != MPQ_HEADER_VERSION {
			bail!("MPQ File header version does not match {:?}", MPQ_HEADER_VERSION);
		}
		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct HashEntry {
	hash_check: [u32; 2],
	lcid: u32,
	block: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct BlockEntry {
	offset: u32,
	size_alloc: u32,
	size_file: u32,
	flags: u32,
}

impl BlockEntry {
	unsafe fn read_from_buf<R: Read + Seek>(reader: &mut BufReader<R>, header: &FileHeader) -> anyhow::Result<Vec<Self>> {
		let block_count = header.block_entries_count as usize;
		let mut blocks: Vec<Self> = Vec::with_capacity(block_count);

		let block_offset = header.block_entries_offset as u64;
		reader.seek(SeekFrom::Start(block_offset))
			.context("Failed to seek to block entries")?;

		for _ in 0..block_count {
			let mut block: Self = mem::zeroed();
			let size = mem::size_of::<Self>();
			let slice = std::slice::from_raw_parts_mut(&mut block as *mut _ as *mut u8, size);
			reader
				.read_exact(slice)
				.context("Failed to read block entry")?;
			blocks.push(block);
		}
		Ok(blocks)
	}
}

#[derive(Debug)]
pub struct Mpq {
	reader: BufReader<File>,
	header: FileHeader,
	blocks: Vec<BlockEntry>,
}

impl Mpq {	
	pub fn open<P: AsRef<Path>>(path: P)  -> anyhow::Result<Self> {
		let file = File::open(path)
			.context("Failed to open file")?;
		let mut reader = BufReader::new(file);
		
		let header = unsafe {
			FileHeader::read_from_buf(&mut reader)?
		};

		/*
		TODO: Decryption
		let blocks = unsafe {
			BlockEntry::read_from_buf(&mut reader, &header)?
		};
		blocks.iter()
			.take(10)
			.for_each(|block| {
				println!("{:?}", block);
			});
		*/

		Ok(Self {
			reader,
			header,
			blocks: Vec::new(),
		})
	}
}

