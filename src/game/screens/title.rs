use glfw::{Window, WindowEvent};

use cgmath::*;

use crate::mpq::*;
use crate::gfx::*;

use crate::game::*;
use crate::game::screen::GameScreen;
use crate::game::anim::{Tween, Frame, Looping};

/*
const FONT_FILENAMES: [&'static str; 5] = [
    // Bad dictionary
    // "ui_art\\font16g.pcx",
    // Bad literal flag
    // "ui_art\\font16s.pcx",
    "ui_art\\font24g.pcx",
    "ui_art\\font24s.pcx",
    "ui_art\\font30g.pcx",
    // Bad literal flag
    // "ui_art\\font30s.pcx",
    "ui_art\\font42g.pcx",
    "ui_art\\font42y.pcx",
];
*/

/// Game title screen
/// First screen after the intro video and before the main menu
#[derive(Debug)]
pub struct TitleScreen {
    title: Texture,
    logo_frames: Vec<Texture>,

    logo_animation: Tween<Frame>,
    fade_animation: Tween<Frame>,
}

impl GameScreen for TitleScreen {
    fn new(archive: &Archive) -> anyhow::Result<Self> { 
        let format = Format::R8g8b8a8_uint;
        let filtering = Filtering::Nearest;

        let title = {
            let file = archive.get_file("ui_art\\title.pcx")?;
            let image = Image::read_pcx(&file, None)?;

            let (width, height) = image.dimensions();
            let pixels = &image.pixels;

            Texture::new(
                width, height, 
                format, filtering,
                pixels
            )?
        };

        let logo_frames = {
            let file = archive.get_file("ui_art\\logo.pcx")?;
            let image = ImageArray::read_pcx(&file, 15, Some(250))?;

            let (width, height) = image.dimensions();

            let mut frames: Vec<Texture> = Vec::with_capacity(15);
            for i in 1..=15 {
                let pixels = image.get(15 - i)?;
                let texture = Texture::new(
                    width, height, 
                    format, filtering,
                    pixels
                )?;
                frames.push(texture)
            }
            frames
        };

        let font = {
            let file = archive.get_file("ui_art\\font24g.pcx")?;
            let image = Image::read_pcx(&file, None)?;

            println!("{:?}", image.dimensions());
        };

        Ok(Self {
            title,
            logo_frames,
            fade_animation: Tween::new(Frame(0), Frame(48), 2.0, Looping::OneShot),
            logo_animation: Tween::new(Frame(0), Frame(15), 1.0, Looping::Loop),
        })
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
        let color_white = Vector4::new(1.0, 1.0, 1.0, 1.0);
        
        batch.sprite(&self.title, Xform2D::position(screen_center), color_white);

        let frame: usize = self.logo_animation.value().into();
        let logo = &self.logo_frames[frame];
        let pos = Vector2::new(screen_center.x, RENDER_HEIGHT as f32 - 182.0);
        batch.sprite(logo, Xform2D::position(pos), color_white);

        if !self.fade_animation.is_done() {
            let fade_alpha = 1.0 - self.fade_animation.percentage();
            batch.aabb(screen_center, screen_size, Vector4::new(0.0, 0.0, 0.0, fade_alpha as f32));
        }
    }
}
