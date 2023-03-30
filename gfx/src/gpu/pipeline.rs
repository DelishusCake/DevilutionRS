use std::ffi::CString;
use std::ops::Deref;
use std::{ptr, str};

use anyhow::{bail, Context};

use gl::types::*;

use crate::{Bindable, Topology};

/// Length, in characters, of the info log for shader and pipeline objects
const INFO_LOG_LEN: usize = 1024;

/// OpenGL shader binding
/// (name, bind point)
pub type ShaderBinding = (&'static str, u32);

/// Wrapper for a shader object
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

/// Wrapper for a graphics/compute pipeline object
/// NOTE: OpenGL terminology is "program"
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
