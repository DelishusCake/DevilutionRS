use anyhow::Context;

use crate::gfx::*;
use crate::mpq::Archive;
use crate::game::file::Image;

/*
https://github.com/diasurgical/devilution/blob/master/DiabloUI/artfont.cpp
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

impl Into<f32> for FontSize {
    fn into(self) -> f32 { 
        match self {
            FontSize::Size16 => 16.0,
            FontSize::Size24 => 24.0,
            FontSize::Size30 => 30.0,
            FontSize::Size42 => 42.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FontColor {
    Grey,
    Silver,
    Yellow,
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

#[derive(Debug)]
pub struct Font {
    size: FontSize,
    color: FontColor,
    pub textures: TextureArray,
}

impl Font {
    pub fn load(archive: &Archive, size: FontSize, color: FontColor) -> anyhow::Result<Self> {
        let (filename_bin, filename_pcx) = get_font_filenames(size, color)
            .context("Font size/color pair is invalid")?;

        let file_bin = archive.get_file(&filename_bin).context("Failed to get binary file from archive")?;
        let file_pcx = archive.get_file(&filename_pcx).context("Failed to get pcx file from archive")?;

        let _contents_bin = {
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
            size,
            color,
            textures
        })
    }

    pub fn get_char_info(&self, c: char) -> (u32, f32) {
        let index = 255 - c as u32;
        let advance = self.size.into();
        (index, advance)
    }
}

fn get_font_filenames(size: FontSize, color: FontColor) -> Option<(String, String)> {
    match (size, color) {
        // Error: Bad Dictionary
        // (FontSize::Size16, FontColor::Grey) => None,
        // Error: Bad literal flag
        // (FontSize::Size16, FontColor::Silver) => None,
        // Error: Bad literal flag
        // (FontSize::Size30, FontColor::Silver) => None,
        // Valid pairs
        (size, color) => {
            let size: i32 = size.into();
            let color: char = color.into();

            let filename_bin = format!("ui_art\\font{}.bin", size);
            let filename_pcx = format!("ui_art\\font{}{}.pcx", size, color);
            Some((filename_bin, filename_pcx))
        },
    }
}