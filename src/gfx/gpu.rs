use std::{ptr, str};
use std::ops::Deref;
use std::mem::size_of;
use std::ffi::{c_void, CString};

use anyhow::{bail, Context};

use gl::types::*;

const INFO_LOG_LEN: usize = 1024;

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
            Topology::Lines     => gl::LINES,
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
            Filtering::Linear  => gl::LINEAR,
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
    R32g32b32a32_float,
}

/// OpenGL Vertex format
/// (member count, type, normalized)
pub type GLVertexFormat = (i32, GLenum, bool);

impl Into<GLVertexFormat> for Format {
    fn into(self) -> GLVertexFormat {
        match self {
            Format::R8_uint            => (1, gl::UNSIGNED_BYTE, false),
            Format::R8g8b8a8_uint      => (4, gl::UNSIGNED_BYTE, false),
            Format::R32g32_float       => (2, gl::FLOAT, false),
            Format::R32g32b32a32_float => (4, gl::FLOAT, false),
        }
    }
}

/// OpenGL texture format
/// (format, type)
pub type GLTextureFormat = (GLenum, GLenum);

impl TryInto<GLTextureFormat> for Format {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<GLTextureFormat> {
        match self {
            Format::R8_uint       => Ok((gl::RED, gl::UNSIGNED_BYTE)),
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

/// Dynamic GPU buffer
#[derive(Debug)]
pub struct DynamicBuffer<T> {
    handle: GLuint,
    target: GLenum,
    data: Vec<T>,
}

impl<T> DynamicBuffer<T>
where
    T: std::clone::Clone,
{
    /// Allocate a new buffer
    pub fn new(
        target: GLenum,
        size: usize,
        initial_data: Option<&[T]>,
    ) -> DynamicBuffer<T> {
        // DYNAMIC_DRAW usage for many writes, many draws 
        let usage = gl::DYNAMIC_DRAW;
        // Calculate the maximum byte size of the buffer
        let max_size = (size * size_of::<T>()) as isize;
        // If initial data was suppplied, convert to a c pointer
        let (data, data_ptr) = match initial_data {
            Some(data) => (data.to_vec(), data.as_ptr() as *const c_void),
            None => (Vec::<T>::with_capacity(size), ptr::null()),
        };
        // Generate buffer handle
        let handle = unsafe {
            let mut handle = 0 as GLuint;
            gl::GenBuffers(1, &mut handle as *mut u32);
            gl::BindBuffer(target, handle);
            gl::BufferData(target, max_size, data_ptr, usage);
            handle
        };
        Self {
            handle,
            target,
            data,
        }
    }

    /// Get the current length of the data in the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Clear the pending buffer data
    /// NOTE: Must be flushed!
    pub fn clear(&mut self) {
        self.data.clear()
    }

    /// Push a value to the pending buffer data
    /// NOTE: Must be flushed!
    pub fn push(&mut self, value: T) -> usize {
        assert!((self.data.len() + 1) <= self.data.capacity());

        let idx = self.data.len();
        self.data.push(value);
        idx
    }

    /// Flush the pending data vector
    pub fn flush(&self) {
        if !self.data.is_empty() {
            let size = (self.data.len() * size_of::<T>()) as isize;
            let data = self.data.as_ptr() as *const c_void;
            unsafe {
                gl::BindBuffer(self.target, self.handle);
                gl::BufferSubData(self.target, 0, size, data);
                gl::BindBuffer(self.target, 0);
            }
        }
    }

    /// Bind a range of the buffer
    /// NOTE: Really only used with uniform buffers
    pub fn bind_range(&self, binding: u32, offset: isize) {
        let size = size_of::<T>() as isize;
        unsafe {
            gl::BindBufferRange(self.target, binding, self.handle, offset, size);
        }
    }
}

impl<T> Bindable for DynamicBuffer<T> {
    /// Bind the buffer
    fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.target, self.handle);
        }
    }

    /// Unbind the buffer
    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.target, 0);
        }
    }
}

impl<T> Drop for DynamicBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.handle as *mut u32);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Texture
{
    pub width: usize,
    pub height: usize,
    pub handle: u32,
}

impl Texture {
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filtering as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filtering as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 
                0, pixel_type as i32, width as i32, height as i32, 
                0, pixel_type as u32, pixel_layout, pixels.as_ptr() as *const c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            handle
        };
        Ok(Self {
            width,
            height,
            handle
        })
    }

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
pub struct VertexArray(u32);

impl VertexArray {
    pub fn new() -> Self {
        let handle = unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            vao
        };
        Self(handle)
    }
}

impl Bindable for VertexArray {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.0);
        }
    }
    fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.0 as *mut u32);
        }
    }
}

/// OpenGL shader binding
/// (name, bind point)
pub type ShaderBinding = (&'static str, u32);

#[derive(Debug)]
pub enum Shader {
    Vertex(GLuint),
    Fragment(GLuint),
}

impl Shader {
    pub fn vertex(code: &str, bindings: Option<&[ShaderBinding]>) -> anyhow::Result<Self> {
        unsafe {
            let handle = compile_shader(gl::VERTEX_SHADER, code, bindings)?;
            Ok(Self::Vertex(handle))
        }
    }
    pub fn fragment(code: &str, bindings: Option<&[ShaderBinding]>) -> anyhow::Result<Self> {
        unsafe {
            let handle = compile_shader(gl::FRAGMENT_SHADER, code, bindings)?;
            Ok(Self::Fragment(handle))
        }
    }
}

impl Deref for Shader {
    type Target = GLuint;

    fn deref(&self) -> &GLuint {
        match self {
            Self::Vertex(handle) | Self::Fragment(handle) => handle,
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        match self {
            Self::Vertex(handle) | Self::Fragment(handle) => unsafe { gl::DeleteShader(*handle) },
        }
    }
}

#[derive(Debug)]
pub struct Pipeline {
    program: u32,
    pub topology: Topology,
}

impl Pipeline {
    pub fn new(topology: Topology, shaders: &[&Shader]) -> anyhow::Result<Self> {
        unsafe {
            // Create the program, bind the shaders, and link
            let program = gl::CreateProgram();
            for shader in shaders {
                gl::AttachShader(program, ***shader);
            }
            gl::LinkProgram(program);
            // If the link failed
            if !get_status(|success: &mut GLint| {
                gl::GetProgramiv(program, gl::LINK_STATUS, success)
            }) {
                // Get the program info log
                let info_log = get_info_log(|len: i32, string: *mut GLchar| {
                    gl::GetProgramInfoLog(program, len, ptr::null_mut(), string)
                })?;
                // Delete the program and bail
                gl::DeleteProgram(program);
                bail!(info_log);
            }
            // Link success, return the pipeline
            Ok(Self { program, topology })
        }
    }
}

impl Bindable for Pipeline {
    fn bind(&self) {
        assert_ne!(self.program, 0);
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

unsafe fn compile_shader(
    shader_type: GLenum,
    code: &str,
    bindings: Option<&[ShaderBinding]>,
) -> anyhow::Result<GLuint> {
    match shader_type {
        gl::VERTEX_SHADER | gl::FRAGMENT_SHADER => {
            // Load the shader string and compile the shader
            let shader_str = CString::new(code.as_bytes())
                .context("Failed to convert shader code to a CString")?;
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(shader, 1, &shader_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // If the compile failed
            if !get_status(|success: &mut GLint| {
                gl::GetShaderiv(shader, gl::COMPILE_STATUS, success)
            }) {
                // Get the shader info log
                let info_log = get_info_log(|len: i32, string: *mut GLchar| {
                    gl::GetShaderInfoLog(shader, len, ptr::null_mut(), string)
                })?;
                // Delete the shader and bail
                gl::DeleteShader(shader);
                bail!(info_log);
            }
            // Set the bindings, if supplied
            if let Some(bindings) = bindings {
                for binding in bindings {
                    let (name, bind_point) = *binding;
                    let index = gl::GetUniformBlockIndex(shader, name.as_ptr() as *const i8);
                    gl::UniformBlockBinding(shader, index, bind_point);
                }
            }
            Ok(shader)
        }
        _ => bail!("Unknown shader type {}", shader_type),
    }
}

unsafe fn get_status<F>(callback: F) -> bool
where
    F: FnOnce(&mut GLint),
{
    let mut success = gl::FALSE as GLint;
    callback(&mut success);
    success == gl::TRUE as GLint
}

unsafe fn get_info_log<F>(callback: F) -> anyhow::Result<String>
where
    F: FnOnce(i32, *mut GLchar),
{
    // Get the info log from OpenGL
    let mut info_log = Vec::with_capacity(INFO_LOG_LEN);
    callback(INFO_LOG_LEN as i32, info_log.as_mut_ptr() as *mut GLchar);
    // Convert to a str
    let info_log_str = str::from_utf8(&info_log)
        .context("Failed to convert shader info log into UTF8 string")?
        .trim_matches(char::from(0));
    Ok(info_log_str.to_string())
}
