use cgmath::*;

use crate::mpq::*;
use crate::gfx::*;

use crate::game::*;
use crate::game::msg::*;
use crate::game::anim::*;
use crate::game::file::*;
use crate::game::screen::{*, town::*};

const COPYRIGHT_TEXT: &'static str = "Copyright © 1996-2001 Blizzard Entertainment";

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

        let font = Font::load(archive, FontSize::Size24, FontColor::Silver)?;

        Ok(Self {
            title,
            logo_frames,
            font,
            fade_animation: OneShotTween::new(Frame(0), Frame(48), 1.0),
            logo_animation: LoopingTween::new(Frame(0), Frame(14), 1.0),
        })
    }

    fn update(&mut self, msg_bus: &mut MsgBus, delta: f64) -> Option<GameScreenName> { 
        self.logo_animation.update(delta);
        self.fade_animation.update(delta);

        while let Some(msg) = msg_bus.dequeue() {
            match msg {
                Msg::Key(_key, _action) => return Some(GameScreenName::Town),
            }
        }

        None
    }

    fn render(&self, batch: &mut Batch) {
        let screen_size = Vector2::new(RENDER_WIDTH as f32, RENDER_HEIGHT as f32);
        let screen_center = screen_size * 0.5;
        let color_white = Vector4::new(1.0, 1.0, 1.0, 1.0);
        
        batch.image(&self.title, Xform2D::position(screen_center), color_white);

        let frame: usize = self.logo_animation.value().into();
        let frame = (self.logo_frames.layers - frame) - 1;
        let pos = Vector2::new(screen_center.x, RENDER_HEIGHT as f32 - 182.0);

        batch.sprite(&self.logo_frames, frame as u32, Xform2D::position(pos), color_white);

        let text_width = self.font.get_width(COPYRIGHT_TEXT);
        let text_offset = Vector2::new(((RENDER_WIDTH - text_width) / 2) as f32, 410.0);
        let char_size = Vector2::new(24.0, 26.0);
        let text_pos = (char_size*0.5) + text_offset;
        for (index, pos) in self.font.render(COPYRIGHT_TEXT, text_pos) {
            batch.sprite(&self.font.textures, index as u32, Xform2D::position(pos), color_white);
        }

        if !self.fade_animation.is_done() {
            let fade_alpha = 1.0 - self.fade_animation.percentage();
            batch.aabb(screen_center, screen_size, Vector4::new(0.0, 0.0, 0.0, fade_alpha as f32));
        }
    }
}
