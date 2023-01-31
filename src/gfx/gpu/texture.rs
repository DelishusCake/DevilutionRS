use std::ffi::c_void;

use gl::types::*;

use crate::gfx::{Format, Filtering, Bindable};

/// Texture object
/// TODO: Utilize a texture queue/manager to allow for async texture creation
#[derive(Debug, Clone, PartialEq)]
pub struct Texture
{
    pub width: usize,
    pub height: usize,
    pub handle: u32,
}

impl Texture {
    /// Create a new texture
    pub fn new(
        width: usize, height: usize, 
        format: Format, filtering: Filtering,
        pixels: &[u8],
    ) -> anyhow::Result<Self> {
        let handle = unsafe {
            let (pixel_type, pixel_layout) = format.try_into()?;
            let filtering: GLenum = filtering.into();

            let mut handle = 0u32;
            gl::GenTextures(1, &mut handle as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, handle);
            {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filtering as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filtering as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

                gl::TexImage2D(gl::TEXTURE_2D, 
                    0, pixel_type as i32, width as i32, height as i32, 
                    0, pixel_type as u32, pixel_layout, pixels.as_ptr() as *const c_void);
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);

            handle
        };
        Ok(Self {
            width,
            height,
            handle
        })
    }

    /// Bind the texture to a texture slot
    /// NOTE: Shader bindings must be set to the texture slot index!
    pub fn bind_at(&self, index: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + index);
            gl::BindTexture(gl::TEXTURE_2D, self.handle);
        }
    }
}

impl Bindable for Texture {
    fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.handle)
        }
    }
    fn unbind(&self) { 
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0)
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle)
        }
    }
}

#[derive(Debug)]
pub struct TextureArray {
    pub width: usize,
    pub height: usize,
    pub layers: usize,
    pub handle: u32,
}

impl TextureArray {
    /// Create a new texture
    pub fn new(
        width: usize, height: usize, layers: usize,
        format: Format, filtering: Filtering,
        pixels: &[u8],
    ) -> anyhow::Result<Self> {
        let handle = unsafe {
            let (pixel_format, pixel_type) = format.try_into()?;
            let filtering: GLenum = filtering.into();

            let mut handle = 0u32;
            gl::GenTextures(1, &mut handle as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, handle);
            {
                gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, filtering as i32);
                gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, filtering as i32);
                gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                
                gl::TexImage3D(gl::TEXTURE_2D_ARRAY, 
                    0, pixel_format as i32, width as i32, height as i32, layers as i32,
                    0, pixel_format as u32, pixel_type, pixels.as_ptr() as *const c_void);
            }
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);

            handle
        };
        Ok(Self {
            width,
            height,
            layers,
            handle
        })
    }

    /// Bind the texture to a texture slot
    /// NOTE: Shader bindings must be set to the texture slot index!
    pub fn bind_at(&self, index: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + index);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.handle);
        }
    }
}

impl Bindable for TextureArray {
    fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.handle)
        }
    }
    fn unbind(&self) { 
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0)
        }
    }
}

impl Drop for TextureArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle)
        }
    }
}
