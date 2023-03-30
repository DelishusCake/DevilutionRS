use std::str::Chars;

use anyhow::Context;

use cgmath::Vector2;

use mpq::Archive;

use gfx::*;

use crate::file::Image;

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
        let (filename_bin, filename_pcx) =
            get_font_filenames(size, color).context("Font size/color pair is invalid")?;

        let bin = {
            let file = archive
                .get_file(&filename_bin)
                .context("Failed to get binary file from archive")?;

            let mut buf = vec![0x0u8; file.size()];
            file.read(&mut buf)?;
            buf
        };

        let textures = {
            let layers = 256;
            let alpha_index = 32;
            let format = Format::R8g8b8a8_uint;
            let filtering = Filtering::Nearest;

            let file = archive
                .get_file(&filename_pcx)
                .context("Failed to get pcx file from archive")?;
            let image = Image::read_pcx(&file, Some(alpha_index))?;

            let (width, height) = image.dimensions();
            let height = height / layers;

            TextureArray::new(width, height, layers, format, filtering, &image.pixels)?
        };

        Ok(Self { bin, textures })
    }

    /// Get the advance, in pixels, of a character in this font
    pub fn get_advance_x(&self, c: char) -> u8 {
        let c = c as usize;
        // If the character has a value stored in the bin file
        if self.bin[c + 2] != 0 {
            // Return it
            self.bin[c + 2]
        } else {
            // Otherwise return the whitespace width
            self.bin[0]
        }
    }

    /// Get the width of a string, if rendered in this font
    pub fn get_width(&self, string: &str) -> u32 {
        string.chars().map(|c| self.get_advance_x(c) as u32).sum()
    }

    pub fn render<'a>(&'a self, string: &'a str, pos: Vector2<f32>) -> FontStringItr {
        let chars = string.chars();
        FontStringItr {
            font: self,
            chars,
            pos,
        }
    }
}

/// Tuple representing a (layer, position) pair for string rendering
pub type FontPoint = (u32, Vector2<f32>);

#[derive(Debug)]
pub struct FontStringItr<'s, 'f> {
    font: &'f Font,
    chars: Chars<'s>,
    pos: Vector2<f32>,
    // TODO: Bounding boxes
}

impl Iterator for FontStringItr<'_, '_> {
    type Item = FontPoint;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next character, or return none if the character iterator is now empty
        let c = self.chars.next()?;
        let (index, advance) = {
            // Clamp character to ascii range
            let index = (c as u32).clamp(0, 255);
            // Invert to layer
            let index = 255 - index;
            // TODO: Newline characters
            // Get the x advancement for this character
            let advance_x = self.font.get_advance_x(c);
            (index, Vector2::new(advance_x as f32, 0.0))
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
