use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use gl::types::*;

use crate::Bindable;

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
    pub fn new(target: GLenum, size: usize, initial_data: Option<&[T]>) -> DynamicBuffer<T> {
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

/// Wrapper for a Vertex Array Object (VAO)
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
