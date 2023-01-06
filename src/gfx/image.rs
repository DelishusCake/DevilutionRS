use std::io::{Result, Error, ErrorKind, Cursor};

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
        if !reader.is_paletted() {
            return Err(Error::new(ErrorKind::InvalidData, "Non-paletted images are not yet supported"));
        }
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
        // NOTE: In a PCX file, the pallete is stored at the very bottom of the image,
        // so this method *must* be called at the end of the image read and consumes the reader
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

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[derive(Debug)]
pub struct ImageArray {
    pub width: usize,
    pub height: usize,
    pub count: usize,
    image: Image,
}

impl ImageArray {
    pub fn read_pcx(file: &File, count: usize, transparency: Option<u8>) -> Result<Self> {
        // Load the entire image from the file
        let image = Image::read_pcx(file, transparency)?;
        // Get the image dimensions
        let (width, height) = image.dimensions();
        // Convert the full image height into the frame height
        // NOTE: Diablo image arrays seem to be stored on the y-axis
        let height = height / count;

        Ok(Self {
            width,
            height,
            count,
            image,
        })
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get(&self, index: usize) -> Result<&[u8]> {
        // Bounds check
        // TODO: This should probably be swapped with a proper index-out-of-bounds error
        if index >= self.count {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Image index out of bounds"));
        }
        // Calculate the pitch of the image
        let bpp = 4;
        let pitch = self.width*bpp;
        // Get the pixel bounds of this frame
        let offset = self.height*index* pitch;
        let limit = self.height*(index+1)* pitch;
        // Return the pixel array
        Ok(&self.image.pixels[offset..limit])
    }
}
