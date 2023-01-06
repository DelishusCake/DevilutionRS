use std::mem::size_of;
use memoffset::offset_of;

use glfw::{WindowHint, OpenGlProfileHint};

use cgmath::*;
use anyhow::Context;

use crate::mpq::Archive;
use crate::gfx::*;

pub const TITLE: &str = "Diablo";
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    pos: Vector2<f32>,
    uv: Vector2<f32>,
    col: Vector4<f32>,
}

pub const LAYOUT: [VertexLayout; 3] = [
    VertexLayout::member(Format::R32g32_float, size_of::<Vertex>(), offset_of!(Vertex, pos)),
    VertexLayout::member(Format::R32g32_float, size_of::<Vertex>(), offset_of!(Vertex, uv)),
    VertexLayout::member(Format::R32g32b32a32_float, size_of::<Vertex>(), offset_of!(Vertex, col)),
];

pub const VERTEX_SHADER: &str = include_str!("gfx/shaders/basic.vert");
pub const FRAGMENT_SHADER: &str = include_str!("gfx/shaders/basic.frag");

#[derive(Debug, Clone)]
#[repr(C)]
struct Uniforms {
    transform: Matrix4<f32>,
}

#[derive(Debug)]
pub struct App {
    // Diablo MPQ archive
    mpq: Archive,
}

impl App {
    pub fn init() -> anyhow::Result<Self> {
        let mpq = Archive::open("data/DIABDAT.MPQ")?;
        Ok(Self {
            mpq
        })
    }
    
    pub fn run(self) -> anyhow::Result<()> {
        use glfw::Context;

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("Failed to initialize GLFW3")?;

        glfw.window_hint(WindowHint::Resizable(true));
        glfw.window_hint(WindowHint::SRgbCapable(true));
        glfw.window_hint(WindowHint::DoubleBuffer(true));
        glfw.window_hint(WindowHint::ContextVersion(3, 3));
        glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(WindowHint::OpenGlDebugContext(cfg!(debug_assertions)));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        
        let (mut window, _events) = glfw
            .create_window(SCREEN_WIDTH, SCREEN_HEIGHT, TITLE, glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window")?;

        window.set_key_polling(true);
        window.make_current();

        gl::load_with(|s| glfw.get_proc_address_raw(s));

        let vertex_shader = Shader::vertex(VERTEX_SHADER, None)?;
        let fragment_shader = Shader::fragment(FRAGMENT_SHADER, None)?;

        let pipeline = Pipeline::new(Topology::Triangles, &[ &vertex_shader, &fragment_shader ])?;

        let (_width, _height, texture) = {
            let title_file = self.mpq.get_file("ui_art\\title.pcx")?;
            let title_image = Image::read_pcx(&title_file, Some(250))?;

            let (width, height) = title_image.dimensions();
            let pixels = &title_image.pixels;

            let texture = Texture::new(
                width, height, 
                Format::R8g8b8a8_uint, 
                Filtering::Nearest,
                pixels
            )?;
            (title_image.width, title_image.height, texture)
        };

        let transform = Matrix4::from_scale(2.0);
        
        let uniforms = Uniforms {
            transform
        };

        let indices = vec![0u16, 1u16, 2u16, 0u16, 3u16, 2u16];

        let vertices = vec![
            Vertex { pos: Vector2::new(-0.5, -0.5), uv: Vector2::new(0.0, 0.0), col: Vector4::new(1.0, 1.0, 1.0, 1.0) },
            Vertex { pos: Vector2::new( 0.5, -0.5), uv: Vector2::new(1.0, 0.0), col: Vector4::new(1.0, 1.0, 1.0, 1.0) },
            Vertex { pos: Vector2::new( 0.5,  0.5), uv: Vector2::new(1.0, 1.0), col: Vector4::new(1.0, 1.0, 1.0, 1.0) },
            Vertex { pos: Vector2::new(-0.5,  0.5), uv: Vector2::new(0.0, 1.0), col: Vector4::new(1.0, 1.0, 1.0, 1.0) },
        ];

        let vertex_buffer = DynamicBuffer::new(gl::ARRAY_BUFFER, vertices.len(), Some(&vertices));
        let index_buffer = DynamicBuffer::new(gl::ELEMENT_ARRAY_BUFFER, indices.len(), Some(&indices));
        let uniform_buffer = DynamicBuffer::new(gl::UNIFORM_BUFFER, indices.len(), Some(&[ uniforms ]));

        let vertex_array = VertexArray::new();
        vertex_array.bind();
        {
            index_buffer.bind();
            vertex_buffer.bind();
            
            VertexLayout::bind(&LAYOUT);

            let offset = 0;
            let binding = 0;
            uniform_buffer.bind_range(binding, offset);
        }
        vertex_array.unbind();

        /*
        let tex_location = unsafe {
            let location = CString::new("u_texture".as_bytes())?;
            gl::GetUniformLocation(program, location.as_ptr())
        };
        */

        while !window.should_close() {
            let aspect_ratio = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
            let window_size = window.get_framebuffer_size();
            let viewport = Viewport::from_window(aspect_ratio, window_size);

            unsafe {
                gl::Enable(gl::SCISSOR_TEST);
                gl::Disable(gl::CULL_FACE);

                gl::Enable(gl::FRAMEBUFFER_SRGB);

                gl::Enable(gl::BLEND);
                gl::BlendEquation(gl::FUNC_ADD);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                gl::Viewport(viewport.x, viewport.y, viewport.w, viewport.h);
                gl::Scissor(viewport.x, viewport.y, viewport.w, viewport.h);

                gl::Clear(gl::COLOR_BUFFER_BIT);    
            }
            
            pipeline.bind();
            texture.bind_at(0);
            // gl::Uniform1i(tex_location, 0);
            {    
                vertex_array.bind();
                unsafe {
                    gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_SHORT, std::ptr::null());
                }
                vertex_array.unbind();
            }
            texture.unbind();
            pipeline.unbind();

            window.swap_buffers();
            glfw.poll_events();
            
            /*
            for (_, event) in glfw::flush_messages(&events) {
                handle_window_event(&mut window, event);
            }
            */
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct Viewport {
    x: i32,
    y: i32,
    w: i32, 
    h: i32, 
}

impl Viewport {
    pub fn from_window(aspect_ratio: f32, window_size: (i32, i32)) -> Self {
        let (width, height) = window_size;
        let mut w = width;
        let mut h = (w as f32 / aspect_ratio + 0.5f32) as i32;
        if h > height {
            h = height;
            w = (height as f32 * aspect_ratio + 0.5f32) as i32;
        }
        let x = (width - w) / 2;
        let y = (height - h) / 2;
        Self { x, y, w, h }
    }
}
