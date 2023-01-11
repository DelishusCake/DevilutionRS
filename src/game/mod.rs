mod anim;
mod title_screen;

pub use anim::*;
pub use title_screen::*;

use glfw::{Window, WindowEvent};

use crate::gfx::Batch;
use crate::mpq::Archive;

pub const RENDER_WIDTH: u32 = 640;
pub const RENDER_HEIGHT: u32 = 480;

pub trait GameScreen {
    fn on_enter(&mut self, archive: &Archive) -> anyhow::Result<()>;
    fn handle_event(&mut self, window: &mut Window, event: &WindowEvent);
    fn update_and_render(&mut self, delta: f64, batch: &mut Batch);
}
