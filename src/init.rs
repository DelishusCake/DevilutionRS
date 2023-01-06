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

        let title = {
            let file = self.mpq.get_file("ui_art\\title.pcx")?;
            let image = Image::read_pcx(&file, None)?;

            let (width, height) = image.dimensions();
            let pixels = &image.pixels;

            let texture = Texture::new(
                width, height, 
                Format::R8g8b8a8_uint, 
                Filtering::Nearest,
                pixels
            )?;
            texture
        };

        let logo_frames = {
            let file = self.mpq.get_file("ui_art\\logo.pcx")?;
            let image = ImageArray::read_pcx(&file, 15, Some(250))?;

            let (width, height) = image.dimensions();

            let mut frames: Vec<Texture> = Vec::with_capacity(15);
            for i in 1..=15 {
                let pixels = image.get(15 - i)?;
                let texture = Texture::new(
                    width, height, 
                    Format::R8g8b8a8_uint, 
                    Filtering::Nearest,
                    pixels
                )?;
                frames.push(texture)
            }
            frames
        };

        let mut batch = Batch::new(1024, 1024); 

        let logo_frame_time = 1.0 / 15.0;

        let mut logo_timer = 0f64;
        let mut logo_frame_index = 0usize; 

        let mut last_time = glfw.get_time();
        while !window.should_close() {
            let now_time = glfw.get_time();
            let delta = now_time - last_time;
            last_time = now_time;

            logo_timer += delta;
            if logo_timer >= logo_frame_time {
                logo_frame_index = (logo_frame_index + 1) % logo_frames.len();
                logo_timer -= logo_frame_time;
            }


            let window_size = window.get_framebuffer_size();
            let aspect_ratio = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
            
            let viewport = Viewport::from_window(aspect_ratio, window_size);
            let projection = {
                let scale_x = window_size.0 as f32 / SCREEN_WIDTH as f32;
                let scale_y = window_size.1 as f32 / SCREEN_HEIGHT as f32;
                let scale = Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0);
                let ortho = ortho(0.0, window_size.0 as f32, window_size.1 as f32, 0.0, -1.0, 1.0);
                ortho*scale
            };

            batch.clear();
            {
                let screen_center = Vector2::new(SCREEN_WIDTH as f32 * 0.5, SCREEN_HEIGHT as f32 * 0.5);
                
                batch.sprite(&title, Xform2D::position(screen_center), Vector4::new(1.0, 1.0, 1.0, 1.0));

                let logo = &logo_frames[logo_frame_index];
                let pos = Vector2::new(SCREEN_WIDTH as f32 * 0.5, SCREEN_HEIGHT as f32 - 182.0);
                batch.sprite(logo, Xform2D::position(pos), Vector4::new(1.0, 1.0, 1.0, 1.0));
            }
            batch.flush(projection);

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

            batch.render(&pipeline);

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
