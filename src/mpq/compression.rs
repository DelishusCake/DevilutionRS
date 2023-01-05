use bitflags::bitflags;

use anyhow::{bail, Context};

bitflags! {
	/// Compression type flags
	struct Compression : u8 {
		const PKWARE = 0x08;
	}
}

pub fn decompress_into(data: &[u8], _out: &mut [u8]) -> anyhow::Result<usize> {
	let _compresion = Compression { bits: data[0] };

	bail!("No compression implemented");
}

pub fn explode_into(data: &[u8], out: &mut [u8]) -> anyhow::Result<usize> {
	let buffer = explode::explode(&data)
		.context("Failed to explode block")?;

	let mut bytes_written = 0usize;
	for (dst, src) in out.into_iter().zip(buffer.iter()) {
		*dst = *src;
		bytes_written += 1;
	}
	Ok(bytes_written)
}
