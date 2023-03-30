use std::io::{Cursor, Result};

use gfx::*;
use mpq::File;

const IMAGE_TEXTURE_FORMAT: Format = Format::R8g8b8a8_uint;
const IMAGE_TEXTURE_FILTERING: Filtering = Filtering::Nearest;

#[derive(Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl Image {
    /// Read a PCX image from an MPQ file, with an optional transparency
    pub fn read_pcx(file: &File, transparency_index: Option<u8>) -> Result<Self> {
        // First, read and decompress the file into a buffer
        let mut file_buf = vec![0x0u8; file.size()];
        file.read(&mut file_buf)?;
        // Create a new PCX file reader over the file buffer
        let mut reader = pcx::Reader::new(Cursor::new(file_buf))?;
        // This is here to catch any images without a palette
        if !reader.is_paletted() {
            panic!("Non-paletted images are not yet supported");
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
        let mut src_image = vec![0x0u8; width * height];
        for y in 0..height {
            let min = y * width;
            let max = (y + 1) * width;
            let row = &mut src_image[min..max];
            reader.next_row_paletted(row)?;
        }
        // Read in the palette for the image
        // NOTE: In a PCX file, the pallete is stored at the very bottom of the image,
        // so this method *must* be called at the end of the image read and consumes the reader
        let mut palette = [0x0u8; 256 * 3];
        if let Some(_len) = reader.palette_length() {
            reader.read_palette(&mut palette)?;
        }
        /*
        NOTE: Test code to find the possible transparency index for an image.
        Transparency is usually 250, but fonts seem to use other indices, depending.

        let mut i = 0;
        while i < 256 {
            let r = palette[i*3 + 0];
            let g = palette[i*3 + 1];
            let b = palette[i*3 + 2];
            if r == 0 && g == 0xFF && b == 0 {
                println!("{:?}", i);
            }
            i = i + 1;
        }
        */
        // Blit to the image pixels based on the palette and transparency
        unsafe {
            if let Some(transparency_index) = transparency_index {
                blit_with_palette_and_transparency(
                    width,
                    height,
                    &palette,
                    transparency_index,
                    &src_image,
                    &mut pixels,
                );
            } else {
                blit_with_palette(width, height, &palette, &src_image, &mut pixels);
            }
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn into_texture(self) -> anyhow::Result<Texture> {
        let (width, height) = self.dimensions();
        Texture::new(
            width,
            height,
            IMAGE_TEXTURE_FORMAT,
            IMAGE_TEXTURE_FILTERING,
            &self.pixels,
        )
    }

    pub fn into_texture_array(self, layers: usize) -> anyhow::Result<TextureArray> {
        let (width, height) = self.dimensions();
        let height = height / layers;

        TextureArray::new(
            width,
            height,
            layers,
            IMAGE_TEXTURE_FORMAT,
            IMAGE_TEXTURE_FILTERING,
            &self.pixels,
        )
    }
}
