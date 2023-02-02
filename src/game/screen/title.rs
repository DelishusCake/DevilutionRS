use glfw::{Window, WindowEvent};

use cgmath::*;

use crate::mpq::*;
use crate::gfx::*;

use crate::game::*;
use crate::game::anim::*;
use crate::game::file::*;
use crate::game::screen::GameScreen;

/// Game title screen
/// First screen after the intro video and before the main menu
#[derive(Debug)]
pub struct TitleScreen {
    title: Texture,
    logo_frames: TextureArray,

    font: Font,

    logo_animation: LoopingTween<Frame>,
    fade_animation: OneShotTween<Frame>,
}

impl GameScreen for TitleScreen {
    fn new(archive: &Archive) -> anyhow::Result<Self> { 
        let format = Format::R8g8b8a8_uint;
        let filtering = Filtering::Nearest;

        let title = {
            let file = archive.get_file("ui_art\\title.pcx")?;
            let image = Image::read_pcx(&file, None)?;

            let (width, height) = image.dimensions();

            Texture::new(
                width, height, 
                format, filtering,
                &image.pixels)?
        };

        let logo_frames = {
            let layers = 15;
            let alpha_index = 250;

            let file = archive.get_file("ui_art\\logo.pcx")?;
            let image = Image::read_pcx(&file, Some(alpha_index))?;

            let (width, height) = image.dimensions();
            let height = height / layers;

            TextureArray::new(
                width, height, layers, 
                format, filtering, 
                &image.pixels)?
        };

        let font = Font::load(archive, FontSize::Size42, FontColor::Grey)?;

        Ok(Self {
            title,
            logo_frames,
            font,
            fade_animation: OneShotTween::new(Frame(0), Frame(48), 1.0),
            logo_animation: LoopingTween::new(Frame(0), Frame(14), 1.0),
        })
    }

    fn handle_event(&mut self, window: &mut Window, event: &WindowEvent) { 
        use glfw::{Key, Action};

        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            },
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
        let frame = (self.logo_frames.layers - frame) - 1;
        let pos = Vector2::new(screen_center.x, RENDER_HEIGHT as f32 - 182.0);

        batch.sprite_layer(&self.logo_frames, frame as u32, Xform2D::position(pos), color_white);

        for (idx, pos) in self.font.render("Test String", screen_center) {
            batch.sprite_layer(&self.font.textures, idx, Xform2D::position(pos), color_white);
        }

        if !self.fade_animation.is_done() {
            let fade_alpha = 1.0 - self.fade_animation.percentage();
            batch.aabb(screen_center, screen_size, Vector4::new(0.0, 0.0, 0.0, fade_alpha as f32));
        }
    }
}
