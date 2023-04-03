use std::io::{Error, ErrorKind, Result};

use bitflags::bitflags;

bitflags! {
    /// Compression type flags
    struct Compression : u8 {
        const PKWARE = 0x08;
    }
}

pub fn decompress_into(data: &[u8], _out: &mut [u8]) -> Result<usize> {
    let _compresion = Compression { bits: data[0] };

    todo!()
}

/// Utilize the PKWare explode algorithm to decompress a byte array into an output buffer
pub fn explode_into(data: &[u8], out: &mut [u8]) -> Result<usize> {
    // Explode the data into a new buffer
    // TODO: Check if this is the fastest way to do this
    // A new allocation on every sector is probably pretty slow
    // it might be more performant with a custom implementation
    let buffer = explode::explode(&data).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to explode block: {}", e),
        )
    })?;
    // Copy into the output buffer
    for (dst, src) in out.into_iter().zip(buffer.iter()) {
        *dst = *src;
    }
    Ok(buffer.len())
}


/// Direct copy one buffer into another
pub fn copy_into(data: &[u8], out: &mut [u8]) -> Result<usize> {
    for (dst, src) in out.into_iter().zip(data.iter()) {
        *dst = *src;
    }
    Ok(data.len())
}
