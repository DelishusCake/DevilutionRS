use std::ffi::c_void;
use std::mem::size_of;

use memoffset::offset_of;

use gl::types::*;

use cgmath::*;

use super::Xform2D;

use super::gpu::*;
use super::material::{Material, MaterialMap};

/// Geometry batching renderer
/// Records draw requests and transforms them into GPU-usable data
#[derive(Debug)]
pub struct Batch {
    // The current ranges to draw
    ranges: Vec<Range>,

    // VBO/VAO objects
    // TODO: Utilize ring buffers to enqueue frames for rendering
    uniforms: DynamicBuffer<Uniforms>,
    indices: DynamicBuffer<u16>,
    vertices: DynamicBuffer<Vertex>,
    vertex_array: VertexArray,
}

impl Batch {
    /// Create a new batch with a specified maximum number of vertices and indices
    pub fn new(max_vertices: usize, max_indices: usize) -> Self {
        // Allocate buffers
        let uniforms: DynamicBuffer<Uniforms> = DynamicBuffer::new(gl::UNIFORM_BUFFER, 1, None);
        let indices: DynamicBuffer<u16> =
            DynamicBuffer::new(gl::ELEMENT_ARRAY_BUFFER, max_indices, None);
        let vertices: DynamicBuffer<Vertex> =
            DynamicBuffer::new(gl::ARRAY_BUFFER, max_vertices, None);

        // Create the vao and bind the vertex layout
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

    // Clear the batch for the next frame of rendering
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.ranges.clear();
    }
    // Flush any recorded draw data (including the projection matrix)
    pub fn flush(&mut self, projection: Matrix4<f32>) {
        self.uniforms.clear();
        self.uniforms.push(Uniforms { projection });
        self.uniforms.flush();

        self.vertices.flush();
        self.indices.flush();
    }

    /// Draw an Axis Aligned Bounding Box (i.e. a non-rotatable, unfilled rectangle)
    /// NOTE: Mostly used for debugging
    pub fn aabb(&mut self, pos: Vector2<f32>, size: Vector2<f32>, color: Vector4<f32>) {
        const INDEX_PATTERN: [usize; 6] = [0, 1, 2, 0, 3, 2];

        let hw = size.x * 0.5;
        let hh = size.y * 0.5;

        let positions = [
            Vector2::new(pos.x - hw, pos.y - hh),
            Vector2::new(pos.x + hw, pos.y - hh),
            Vector2::new(pos.x + hw, pos.y + hh),
            Vector2::new(pos.x - hw, pos.y + hh),
        ];

        self.push_range(
            Topology::Triangles,
            Material::Color,
            0,
            |vertices, indices| {
                let mut quad_indices = [0u16; 6];
                for (i, pos) in positions.iter().enumerate() {
                    let vertex = Vertex {
                        pos: *pos,
                        uv: Vector3::zero(),
                        col: color,
                    };
                    quad_indices[i] = vertices.push(vertex) as u16;
                }
                for offset in INDEX_PATTERN {
                    indices.push(quad_indices[offset]);
                }
            },
        );
    }

    /// Draw a textured, colored quad using the specified transform
    /// TODO: Support specifying the portion of the texture to draw
    pub fn image(&mut self, texture: &Texture, xform: Xform2D, color: Vector4<f32>) {
        const INDEX_PATTERN: [usize; 6] = [0, 1, 2, 0, 3, 2];

        let size = (texture.width as f32, texture.height as f32);
        let _i_size = vec2(1.0 / size.0, 1.0 / size.1);

        let (s0, t0, s1, t1) = (0.0, 0.0, 1.0, 1.0);
        let (_x, _y, w, h) = (0.0, 0.0, size.0, size.1);

        let hw = w * 0.5;
        let hh = h * 0.5;

        let verts = [
            Vertex {
                pos: xform * vec2(-hw, -hh),
                uv: vec3(s0, t1, 0.0),
                col: color,
            },
            Vertex {
                pos: xform * vec2(hw, -hh),
                uv: vec3(s1, t1, 0.0),
                col: color,
            },
            Vertex {
                pos: xform * vec2(hw, hh),
                uv: vec3(s1, t0, 0.0),
                col: color,
            },
            Vertex {
                pos: xform * vec2(-hw, hh),
                uv: vec3(s0, t0, 0.0),
                col: color,
            },
        ];

        self.push_range(
            Topology::Triangles,
            Material::Textured,
            texture.handle,
            |vertices, indices| {
                let mut sprite_indices = [0u16; 6];
                for (i, v) in verts.iter().enumerate() {
                    sprite_indices[i] = vertices.push(*v) as u16;
                }
                for offset in INDEX_PATTERN {
                    indices.push(sprite_indices[offset]);
                }
            },
        );
    }

    /// Draw a textured, colored quad using the specified transform
    /// TODO: Support specifying the portion of the texture to draw
    pub fn sprite(
        &mut self,
        texture: &TextureArray,
        layer: u32,
        xform: Xform2D,
        color: Vector4<f32>,
    ) {
        const INDEX_PATTERN: [usize; 6] = [0, 1, 2, 0, 3, 2];

        let size = (texture.width as f32, texture.height as f32);
        let _i_size = vec2(1.0 / size.0, 1.0 / size.1);

        let l = layer as f32;
        let (s0, t0, s1, t1) = (0.0, 0.0, 1.0, 1.0);
        let (_x, _y, w, h) = (0.0, 0.0, size.0, size.1);

        let hw = w * 0.5;
        let hh = h * 0.5;

        let verts = [
            Vertex {
                pos: xform * vec2(-hw, -hh),
                uv: vec3(s0, t1, l),
                col: color,
            },
            Vertex {
                pos: xform * vec2(hw, -hh),
                uv: vec3(s1, t1, l),
                col: color,
            },
            Vertex {
                pos: xform * vec2(hw, hh),
                uv: vec3(s1, t0, l),
                col: color,
            },
            Vertex {
                pos: xform * vec2(-hw, hh),
                uv: vec3(s0, t0, l),
                col: color,
            },
        ];

        self.push_range(
            Topology::Triangles,
            Material::LayeredTexture,
            texture.handle,
            |vertices, indices| {
                let mut sprite_indices = [0u16; 6];
                for (i, v) in verts.iter().enumerate() {
                    sprite_indices[i] = vertices.push(*v) as u16;
                }
                for offset in INDEX_PATTERN {
                    indices.push(sprite_indices[offset]);
                }
            },
        );
    }

    /// Render the batched geometry to the screen
    /// NOTE: `flush` must be called before rendering
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
    fn push_range<F>(
        &mut self,
        topology: Topology,
        material: Material,
        texture: u32,
        mut draw_fn: F,
    ) where
        F: FnMut(&mut DynamicBuffer<Vertex>, &mut DynamicBuffer<u16>),
    {
        // Get the current offset
        let offset = self.indices.len();
        // Get the range to draw to
        let mut range = {
            // If the range list is empty, or the current range doesn't match draw parameters
            if self.ranges.is_empty()
                || self
                    .ranges
                    .last()
                    .unwrap()
                    .should_change(texture, topology, material)
            {
                // Push a new range
                let range = Range {
                    texture,
                    topology,
                    material,
                    offset,
                    count: 0,
                };
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

/// Vertex structure for the Batch object
#[derive(Debug, Copy, Clone)]
struct Vertex {
    pos: Vector2<f32>,
    uv: Vector3<f32>,
    col: Vector4<f32>,
}

/// Layout descriptor for the vertex structure
const VERTEX_LAYOUT: [VertexLayout; 3] = [
    VertexLayout {
        format: Format::R32g32_float,
        stride: size_of::<Vertex>(),
        offset: offset_of!(Vertex, pos) as usize,
    },
    VertexLayout {
        format: Format::R32g32b32_float,
        stride: size_of::<Vertex>(),
        offset: offset_of!(Vertex, uv) as usize,
    },
    VertexLayout {
        format: Format::R32g32b32a32_float,
        stride: size_of::<Vertex>(),
        offset: offset_of!(Vertex, col) as usize,
    },
];

/// Range object
/// The Batch compresses rendering into as few draw calls as possible.
/// Each draw call is represented by a Range object, describing the material/topolgy/and offset into the index buffer
#[derive(Debug)]
struct Range {
    texture: u32,
    topology: Topology,
    material: Material,

    offset: usize,
    count: usize,
}

impl Range {
    /// Does this range match a pending draw call?
    fn should_change(&self, texture: u32, topology: Topology, material: Material) -> bool {
        self.texture != texture || self.topology != topology || self.material != material
    }

    /// Render the range
    /// NOTE: Should only be called within the render method of the batch
    fn render(&self, format: GLenum, materials: &MaterialMap) {
        // Get the pipeline to use
        // NOTE: Unwrap here is okay, it's better to just crash if the draw call is invalid
        let pipeline = materials.get(self.topology, self.material).unwrap();
        pipeline.bind();
        unsafe {
            // Get the OpenGL pipeline topology
            let topology: GLenum = self.topology.into();
            // Bind (or unbind) the current texture handle
            gl::ActiveTexture(gl::TEXTURE0 + 0);
            match self.material {
                Material::Textured => {
                    gl::BindTexture(gl::TEXTURE_2D, self.texture);
                }
                Material::LayeredTexture => {
                    gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
                }
                _ => {}
            }
            // Calculate byte offset to indices and convert to void pointer
            let offset = offset_ptr::<i16>(self.offset);
            gl::DrawElements(topology, self.count as i32, format, offset);
            match self.material {
                Material::Textured => {
                    gl::BindTexture(gl::TEXTURE_2D, 0);
                }
                Material::LayeredTexture => {
                    gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
                }
                _ => {}
            }
        }
        pipeline.unbind();
    }
}

/// Uniform structure for batch rendering
/// NOTE: Only one uniform buffer per batch
#[derive(Clone, Debug)]
#[repr(C)]
struct Uniforms {
    projection: Matrix4<f32>,
}

/// Get the pointer-equivalent of an offset into an array of T
/// NOTE: OpenGL likes to use void pointers for offsets for some reason
/// This is fine in C, where pointers are integers, but really annoying in Rust
fn offset_ptr<T>(value: usize) -> *const c_void {
    let byte_offset = value * size_of::<T>();
    byte_offset as *const c_void
}
