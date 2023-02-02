use std::str::Chars;

use anyhow::Context;

use cgmath::Vector2;

use crate::gfx::*;
use crate::mpq::Archive;
use crate::game::file::Image;

/*
NOTES:
https://github.com/diasurgical/devilution/blob/master/DiabloUI/artfont.cpp

Bin File:
    258 bytes = 1 byte for default width [0] + 1 byte newline height [1] + 256 bytes char widths

Font Filenames in DIABDAT.MPQ:
    ui_art\\font16.bin
    ui_art\\font16g.pcx 
        File error on load: Bad dictionary
    ui_art\\font16s.pcx
        File error on load: Bad literal flag
    ui_art\\font24.bin
    ui_art\\font24g.pcx
    ui_art\\font24s.pcx
    ui_art\\font30.bin
    ui_art\\font30g.pcx
    ui_art\\font30s.pcx
        File error on load: Bad literal flag
    ui_art\\font42.bin
    ui_art\\font42g.pcx
    ui_art\\font42y.pcx
*/

#[derive(Debug, Copy, Clone)]
pub enum FontSize {
    Size16,
    Size24,
    Size30,
    Size42,
}

#[derive(Debug, Copy, Clone)]
pub enum FontColor {
    Grey,
    Silver,
    Yellow,
}

#[derive(Debug)]
pub struct Font {
    bin: Vec<u8>,
    pub textures: TextureArray,
}

impl Font {
    pub fn load(archive: &Archive, size: FontSize, color: FontColor) -> anyhow::Result<Self> {
        let (filename_bin, filename_pcx) = get_font_filenames(size, color)
            .context("Font size/color pair is invalid")?;

        let file_bin = archive.get_file(&filename_bin).context("Failed to get binary file from archive")?;
        let file_pcx = archive.get_file(&filename_pcx).context("Failed to get pcx file from archive")?;

        let bin = {
            let mut buf = vec![0x0u8; file_bin.size()];
            file_bin.read(&mut buf)?;
            buf
        };

        let textures = {
            let layers = 256;
            let alpha_index = 32;
            let format = Format::R8g8b8a8_uint;
            let filtering = Filtering::Nearest;
            
            let image = Image::read_pcx(&file_pcx, Some(alpha_index))?;

            let (width, height) = image.dimensions();
            let height = height / layers;

            TextureArray::new(
                width, height, layers, 
                format, filtering, 
                &image.pixels)?
        };

        Ok(Self {
            bin,
            textures
        })
    }

    pub fn get_width(&self, string: &str) -> u32 {
        let mut w = 0;
        for c in string.chars() {
            let c = c as usize;
            let advance_x = if self.bin[c + 2] != 0 {
                self.bin[c + 2]
            } else {
                self.bin[0]
            };
            w = w + (advance_x as u32);
        }
        w
    }

    pub fn render<'a>(&'a self, string: &'a str, pos: Vector2<f32>) -> FontStringItr {
        FontStringItr {
            chars: string.chars(),
            font: self,
            pos,
        }
    }
}

#[derive(Debug)]
pub struct FontStringItr<'s, 'f> {
    font: &'f Font,
    chars: Chars<'s>,
    pos: Vector2<f32>,
    // TODO: Bounding boxes
}

impl Iterator for FontStringItr<'_, '_> {
    type Item = (u32, Vector2<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next character, or return none if the character iterator is now empty
        let c = self.chars.next()?;
        // Get the character as a u32
        let c = c as u32;
        // Match the character to handle non-ASCII characters
        let (index, advance) = match c {
            // ASCII character range
            0..=255 => {
                // Get the index into the texture array for this character
                let index = 255 - c as u32;
                // TODO: Newline characters
                // Get the x advancement for this character
                let advance_x = {
                    // Lookup the character width in the bin file
                    let char_w = self.font.bin[(c as u32 + 2) as usize];
                    if char_w != 0 {
                        char_w as i32
                    } else {
                        // No character width stored, use the default width
                        self.font.bin[0] as i32
                    }
                };
                (index, Vector2::new(advance_x as f32, 0.0))
            },
            _ => {
                let index = 255;
                let advance_x = self.font.bin[0] as i32;
                (index, Vector2::new(advance_x as f32, 0.0))
           },
        };
        // Get the current position to return
        let current_pos = self.pos.clone();
        // Advance by the character advancement
        self.pos += advance;
        // Return the image index and current position
        Some((index, current_pos))
    }
}

fn get_font_filenames(size: FontSize, color: FontColor) -> Option<(String, String)> {
    let size: i32 = size.into();
    let color: char = color.into();

    let filename_bin = format!("ui_art\\font{}.bin", size);
    let filename_pcx = format!("ui_art\\font{}{}.pcx", size, color);
    Some((filename_bin, filename_pcx))
}

impl Into<i32> for FontSize {
    fn into(self) -> i32 { 
        match self {
            FontSize::Size16 => 16,
            FontSize::Size24 => 24,
            FontSize::Size30 => 30,
            FontSize::Size42 => 42,
        }
    }
}

impl Into<char> for FontColor {
    fn into(self) -> char {
        match self {
            FontColor::Grey => 'g',
            FontColor::Silver => 's',
            FontColor::Yellow => 'y',
        }
    }
}
