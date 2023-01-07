use glfw::{Window, WindowEvent};

use cgmath::*;

use crate::mpq::*;
use crate::gfx::*;
use crate::game::*;

#[derive(Debug)]
pub struct TitleScreen {
    title_image: Option<Texture>,
    logo_frames: Option<Vec<Texture>>,

    logo_animation: Animation,
    fade_animation: Animation,
}

impl TitleScreen {
    pub fn new() -> Self {
        Self {
            title_image: None,
            logo_frames: None,
            fade_animation: Animation::new(24, 48, AnimationType::OneShot),
            logo_animation: Animation::new(15, 15, AnimationType::Looping),
        }
    }
}

impl GameScreen for TitleScreen {
    fn on_enter(&mut self, archive: &Archive) -> anyhow::Result<()> { 
        self.title_image = {
            let file = archive.get_file("ui_art\\title.pcx")?;
            let image = Image::read_pcx(&file, None)?;

            let (width, height) = image.dimensions();
            let pixels = &image.pixels;

            let texture = Texture::new(
                width, height, 
                Format::R8g8b8a8_uint, 
                Filtering::Linear,
                pixels
            )?;
            Some(texture)
        };

        self.logo_frames = {
            let file = archive.get_file("ui_art\\logo.pcx")?;
            let image = ImageArray::read_pcx(&file, 15, Some(250))?;

            let (width, height) = image.dimensions();

            let mut frames: Vec<Texture> = Vec::with_capacity(15);
            for i in 1..=15 {
                let pixels = image.get(15 - i)?;
                let texture = Texture::new(
                    width, height, 
                    Format::R8g8b8a8_uint, 
                    Filtering::Linear,
                    pixels
                )?;
                frames.push(texture)
            }
            Some(frames)
        };

        Ok(())
    }
    fn handle_event(&mut self, window: &mut Window, event: &WindowEvent) { 
        use glfw::{Key, Action};

        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {},
        }
    }
    fn update_and_render(&mut self, delta: f64, batch: &mut Batch) { 
        self.logo_animation.update(delta);
        self.fade_animation.update(delta);

        let screen_size = Vector2::new(RENDER_WIDTH as f32, RENDER_HEIGHT as f32);
        let screen_center = screen_size * 0.5;
        
        batch.sprite(self.title_image.as_ref().unwrap(), Xform2D::position(screen_center), Vector4::new(1.0, 1.0, 1.0, 1.0));

        let logo = &self.logo_frames.as_ref().unwrap()[self.logo_animation.index()];
        let pos = Vector2::new(screen_center.x, RENDER_HEIGHT as f32 - 182.0);
        batch.sprite(logo, Xform2D::position(pos), Vector4::new(1.0, 1.0, 1.0, 1.0));

        if !self.fade_animation.is_done() {
            let fade_alpha = 1.0 - self.fade_animation.percentage();
            batch.aabb(screen_center, screen_size, Vector4::new(0.0, 0.0, 0.0, fade_alpha));
        }
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
