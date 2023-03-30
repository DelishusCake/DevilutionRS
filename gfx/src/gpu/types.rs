use std::ffi::c_void;

use anyhow::bail;

use gl::types::*;

/// Trait for GPU resources that must be bound/unbound to be used
pub trait Bindable {
    fn bind(&self);
    fn unbind(&self);
}

/// Geometric topology
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Topology {
    Lines,
    Triangles,
}

impl Into<GLenum> for Topology {
    fn into(self) -> GLenum {
        match self {
            Topology::Lines => gl::LINES,
            Topology::Triangles => gl::TRIANGLES,
        }
    }
}

/// Texture filtering enum
#[derive(Debug, Copy, Clone)]
pub enum Filtering {
    Linear,
    Nearest,
}

impl Into<GLenum> for Filtering {
    fn into(self) -> GLenum {
        match self {
            Filtering::Linear => gl::LINEAR,
            Filtering::Nearest => gl::NEAREST,
        }
    }
}

/// GPU format used for both vertex and texture types
/// Non-camel case types are used to make reading easier
/// Format: <channel><size in bits>..._type
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Format {
    R8_uint,
    R8g8b8a8_uint,
    R32g32_float,
    R32g32b32_float,
    R32g32b32a32_float,
}

/// OpenGL Vertex format
/// (member count, type, normalized)
pub type GLVertexFormat = (i32, GLenum, bool);

impl Into<GLVertexFormat> for Format {
    fn into(self) -> GLVertexFormat {
        match self {
            Format::R8_uint => (1, gl::UNSIGNED_BYTE, false),
            Format::R8g8b8a8_uint => (4, gl::UNSIGNED_BYTE, false),
            Format::R32g32_float => (2, gl::FLOAT, false),
            Format::R32g32b32_float => (3, gl::FLOAT, false),
            Format::R32g32b32a32_float => (4, gl::FLOAT, false),
        }
    }
}

/// OpenGL texture format
/// (format, type, internal_format)
pub type GLTextureFormat = (GLenum, GLenum);

impl TryInto<GLTextureFormat> for Format {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<GLTextureFormat> {
        match self {
            Format::R8_uint => Ok((gl::RED, gl::UNSIGNED_BYTE)),
            Format::R8g8b8a8_uint => Ok((gl::RGBA, gl::UNSIGNED_BYTE)),
            _ => bail!("This format is invalid for textures {:?}", self),
        }
    }
}

/// Vertex layout struct
/// Used to describe vertex formats and layouts
#[derive(Debug)]
pub struct VertexLayout {
    pub format: Format,
    pub stride: usize,
    pub offset: usize,
}

impl VertexLayout {
    pub const fn member(format: Format, stride: usize, offset: usize) -> Self {
        Self {
            format,
            stride,
            offset,
        }
    }

    pub fn bind(layout: &[VertexLayout]) {
        unsafe {
            for (index, entry) in layout.iter().enumerate() {
                // Convert format to OpenGL values
                let (size, format, normalized) = entry.format.into();
                // Enable and set the vertex attrib pointer
                gl::EnableVertexAttribArray(index as u32);
                gl::VertexAttribPointer(
                    index as u32,
                    size,
                    format,
                    normalized as u8,
                    entry.stride as i32,
                    entry.offset as *const c_void,
                );
            }
        }
    }
}
