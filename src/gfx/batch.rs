use std::mem::size_of;
use std::ffi::c_void;

use memoffset::offset_of;

use gl::types::*;

use cgmath::*;

use super::Xform2D;

use super::gpu::*;
use super::material::{MaterialMap, Material};

#[derive(Debug, Copy, Clone)]
struct Vertex {
    pos: Vector2<f32>,
    uv:  Vector2<f32>,
    col: Vector4<f32>,
}

const VERTEX_LAYOUT: [VertexLayout; 3] = 
[
    VertexLayout { format: Format::R32g32_float, stride: size_of::<Vertex>(), offset: offset_of!(Vertex, pos) as usize },
    VertexLayout { format: Format::R32g32_float, stride: size_of::<Vertex>(), offset: offset_of!(Vertex, uv) as usize },
    VertexLayout { format: Format::R32g32b32a32_float, stride: size_of::<Vertex>(), offset: offset_of!(Vertex, col) as usize },
];

#[derive(Debug)]
struct Range {
    texture: u32,
    topology: Topology,
    material: Material,

    offset: usize,
    count: usize,
}

impl Range {
    fn should_change(&self, texture: u32, topology: Topology, material: Material) -> bool {
        self.texture != texture || self.topology != topology || self.material != material
    }
    fn render(&self, format: GLenum, materials: &MaterialMap) {
        let pipeline = materials.get(self.topology, self.material).unwrap();
        pipeline.bind();
        unsafe {
            // Get the OpenGL pipeline topology
            let topology: GLenum = self.topology.into();
            // Bind (or unbind) the current texture handle
            gl::ActiveTexture(gl::TEXTURE0 + 0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            // Calculate byte offset to indices and convert to void pointer
            let offset = offset_ptr::<i16>(self.offset);
            gl::DrawElements(topology, self.count as i32, format, offset);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        pipeline.unbind();
    }
}

/// Get the pointer-equivalent of an offset into an array of T
/// NOTE: OpenGL likes to use void pointers for offsets for some reason
/// This is fine in C, where pointers are integers, but really annoying in Rust
fn offset_ptr<T>(value: usize) -> *const c_void {
    let byte_offset = value * size_of::<T>();
    byte_offset as *const c_void
}

#[derive(Clone, Debug)]
#[repr(C)]
struct Uniforms {
    projection: Matrix4<f32>,
}

#[derive(Debug)]
pub struct Batch {
    ranges: Vec<Range>,

    uniforms: DynamicBuffer<Uniforms>,
    indices: DynamicBuffer<u16>,
    vertices: DynamicBuffer<Vertex>,
    vertex_array: VertexArray,
}

impl Batch {
    pub fn new(max_vertices: usize, max_indices: usize) -> Self {
        let uniforms: DynamicBuffer<Uniforms> = DynamicBuffer::new(gl::UNIFORM_BUFFER, 1, None);
        let indices: DynamicBuffer<u16> =  DynamicBuffer::new(gl::ELEMENT_ARRAY_BUFFER, max_indices, None);
        let vertices: DynamicBuffer<Vertex> = DynamicBuffer::new(gl::ARRAY_BUFFER, max_vertices, None);

        let vertex_array = {
            let vao = VertexArray::new();
            vao.bind();
            {
                vertices.bind();
                VertexLayout::bind(&VERTEX_LAYOUT);
                
                indices.bind();
            }
            vao.unbind();
            vao
        };

        Self {
            ranges: Vec::new(),
            uniforms,
            indices,
            vertices,
            vertex_array,
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.ranges.clear();
    }

    pub fn flush(&mut self, projection: Matrix4<f32>) {
        self.uniforms.clear();
        self.uniforms.push(Uniforms {
            projection,
        });
        self.uniforms.flush();

        self.vertices.flush();
        self.indices.flush();
    }

    pub fn aabb(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Vector4<f32>) {
        const INDEX_PATTERN: [usize;6] = [0, 1, 2, 0, 3, 2];

        let hw = size.x*0.5;
        let hh = size.y*0.5;

        let positions = [
            Vector2::new(pos.x - hw, pos.y - hh),
            Vector2::new(pos.x + hw, pos.y - hh),
            Vector2::new(pos.x + hw, pos.y + hh),
            Vector2::new(pos.x - hw, pos.y + hh),
        ];

        self.push_range(Topology::Triangles, Material::Color, 0, |vertices, indices| {
            let mut quad_indices = [0u16; 6];
            for (i, pos) in positions.iter().enumerate() {
                let vertex = Vertex {
                    pos: *pos,
                    uv: Vector2::zero(),
                    col: color,
                };
                quad_indices[i] = vertices.push(vertex) as u16; 
            }
            for offset in INDEX_PATTERN {
                indices.push(quad_indices[offset]);
            }
        });
    }

    pub fn sprite(&mut self, texture: &Texture, xform: Xform2D, color: Vector4<f32>) {
        const INDEX_PATTERN: [usize;6] = [0, 1, 2, 0, 3, 2];
        
        let size = (texture.width as f32, texture.height as f32);
        let _i_size = vec2(1.0 / size.0, 1.0 / size.1);

        let (s0,t0,s1,t1) = (0.0,0.0,1.0,1.0);
        let (_x,_y,w,h) = (0.0,0.0,size.0,size.1);

        let hw = w*0.5;
        let hh = h*0.5;
        
        let verts = [
            Vertex{ pos: xform.apply(vec2(-hw, -hh)), uv: vec2(s0, t1), col: color },
            Vertex{ pos: xform.apply(vec2( hw, -hh)), uv: vec2(s1, t1), col: color },
            Vertex{ pos: xform.apply(vec2( hw,  hh)), uv: vec2(s1, t0), col: color },
            Vertex{ pos: xform.apply(vec2(-hw,  hh)), uv: vec2(s0, t0), col: color },
        ];

        self.push_range(Topology::Triangles, Material::Textured, texture.handle, |vertices, indices| {
            let mut sprite_indices = [0u16; 6];
            for (i, v) in verts.iter().enumerate() {
                sprite_indices[i] = vertices.push(*v) as u16; 
            }
            for offset in INDEX_PATTERN {
                indices.push(sprite_indices[offset]);
            }
        });
    }

    pub fn render(&self, materials: &MaterialMap) {
        // GL format for the index buffer
        let index_format = gl::UNSIGNED_SHORT;

        // Bind the vertex format
        self.vertex_array.bind();
        // Bind the draw buffers
        self.indices.bind();
        self.vertices.bind();
        // Bind the uniform range
        let offset = 0;
        let binding = 0;
        self.uniforms.bind_range(binding, offset);

        // For each range, bind the pipeline and issue the draw call
        for range in &self.ranges {
            range.render(index_format, materials);
        }
        self.uniforms.unbind();
        self.vertex_array.unbind();
    }

    #[inline]
    fn push_range<F>(&mut self, topology: Topology, material: Material, texture: u32, mut draw_fn: F)
    where 
        F: FnMut(&mut DynamicBuffer<Vertex>, &mut DynamicBuffer<u16>) 
    {
        // Get the current offset
        let offset = self.indices.len();
        // Get the range to draw to
        let mut range = {
            // If the range list is empty, or the current range doesn't match draw parameters
            if self.ranges.is_empty() || self.ranges.last().unwrap().should_change(texture, topology, material) {
                // Push a new range
                let range = Range { texture, topology, material, offset, count: 0 };
                self.ranges.push(range);
            }
            // Return the last range in the list
            self.ranges.last_mut().unwrap()
        };
        // Execute the draw function
        draw_fn(&mut self.vertices, &mut self.indices);
        // Add the new indices to the range count
        range.count += self.indices.len() - offset;
    }
}
