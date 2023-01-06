use std::io::{Result, Cursor};

use crate::mpq::File;
use crate::gfx::util::*;

#[derive(Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl Image {
    /// Read a PCX image from an MPQ file, with an optional transparency
    pub fn read_pcx(file: &File, transparency: Option<u8>) -> Result<Self> {
        // First, read and decompress the file into a buffer
        let mut file_buf = vec![0x0u8; file.size()];
        file.read(&mut file_buf)?;
        // Create a new PCX file reader over the file buffer
        let mut reader = pcx::Reader::new(Cursor::new(file_buf))?;
        // This is here to catch any images without a palette
        assert!(reader.is_paletted(), "Update read_pcx to support non-paletted images!");
        // Get the dimensions of the image
        let (width, height) = reader.dimensions();
        // Output as an RGBA image (4 bytes per pixel) 
        let bpp = 4;
        // Reinterpret the dimensions as usize (makes the math easier)
        let width = width as usize;
        let height = height as usize;
        // Allocate the pixel buffer for the image
        let mut pixels = vec![0x0u8; width * height * bpp];
        // Read in the full source image
        // This will be 1 byte per pixel with each pixel as an index into the palette 
        let mut row = vec![0x0u8; width];
        let mut src_image: Vec<u8> = Vec::with_capacity(width*height);
        for _ in 0..height {
            reader.next_row_paletted(&mut row)?;
            src_image.extend_from_slice(&row);
        }
        // Read in the palette for the image
        let mut palette = [0x0u8; 256*3];
        if let Some(_len) = reader.palette_length() {
            reader.read_palette(&mut palette)?;
        }

        // Blit to the image pixels based on the palette and transparency
        if let Some(transparency) = transparency {
            blit_with_palette_and_transparency(
                width, height, 
                &palette, transparency,
                &src_image, &mut pixels);
        } else {
            blit_with_palette(width, height, &palette, &src_image, &mut pixels);
        }

        Ok(Self {
            width, 
            height, 
            pixels
        })
    }
}
