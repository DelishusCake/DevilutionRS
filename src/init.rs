use glfw::{WindowHint, OpenGlProfileHint};

use cgmath::*;
use anyhow::Context;

use crate::mpq::Archive;
use crate::gfx::*;

pub const TITLE: &str = "Diablo";
pub const SCREEN_WIDTH: u32 = 1920 >> 1;
pub const SCREEN_HEIGHT: u32 = 1080 >> 1;

pub const RENDER_WIDTH: u32 = 640;
pub const RENDER_HEIGHT: u32 = 480;

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

        let mut fade_animation = Animation::new(24, 48, AnimationType::OneShot);
        let mut logo_animation = Animation::new(24, 15, AnimationType::Looping);

        let mut batch = Batch::new(1024, 1024); 

        let materials = MaterialMap::new()?;

        let mut last_time = glfw.get_time();
        while !window.should_close() {
            let now_time = glfw.get_time();
            let delta = now_time - last_time;
            last_time = now_time;

            logo_animation.update(delta);
            fade_animation.update(delta);

            let window_size = window.get_framebuffer_size();
            let aspect_ratio = RENDER_WIDTH as f32 / RENDER_HEIGHT as f32;
            
            let viewport = Viewport::from_window(aspect_ratio, window_size);
            let projection = {
                let scale_x = window_size.0 as f32 / RENDER_WIDTH as f32;
                let scale_y = window_size.1 as f32 / RENDER_HEIGHT as f32;
                let scale = Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0);
                let ortho = ortho(0.0, window_size.0 as f32, window_size.1 as f32, 0.0, -1.0, 1.0);
                ortho*scale
            };

            batch.clear();
            {
                let screen_size = Vector2::new(RENDER_WIDTH as f32, RENDER_HEIGHT as f32);
                let screen_center = screen_size * 0.5;
                
                batch.sprite(&title, Xform2D::position(screen_center), Vector4::new(1.0, 1.0, 1.0, 1.0));

                let logo = &logo_frames[logo_animation.index()];
                let pos = Vector2::new(screen_center.x, RENDER_HEIGHT as f32 - 182.0);
                batch.sprite(logo, Xform2D::position(pos), Vector4::new(1.0, 1.0, 1.0, 1.0));

                if !fade_animation.is_done() {
                    let fade_color = Vector4::new(0.0, 0.0, 0.0, 1.0 - fade_animation.percentage());
                    batch.aabb(screen_center, screen_size, fade_color);
                }
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

            batch.render(&materials);

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

#[derive(Debug)]
enum AnimationType {
    OneShot,
    Looping,
}

#[derive(Debug)]
struct Animation {
    time: f64,
    frame_time: f64,
    max_frames: usize,
    current_frame: usize,
    anim_type: AnimationType,
}

impl Animation {
    pub fn new(frame_rate: u32, max_frames: usize, anim_type: AnimationType) -> Self {
        Self {
            time: 0.0,
            anim_type,
            max_frames,
            frame_time: (1.0 / frame_rate as f64),
            current_frame: 0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.time += dt;
        if self.time >= self.frame_time {
            self.current_frame = match self.anim_type {
                AnimationType::OneShot => usize::min(self.current_frame + 1, self.max_frames),
                AnimationType::Looping => (self.current_frame + 1) % self.max_frames,
            };
            self.time -= self.frame_time;
        }
    }

    pub fn index(&self) -> usize {
        self.current_frame
    }

    pub fn percentage(&self) -> f32 {
        self.current_frame as f32 / self.max_frames as f32
    }

    pub fn is_done(&self) -> bool {
        match self.anim_type {
            AnimationType::OneShot => self.current_frame == self.max_frames,
            AnimationType::Looping => false,
        }
    }
}
