use glfw::{Window, WindowEvent};

use crate::gfx::Batch;
use crate::mpq::Archive;

pub trait GameScreen {
    fn new(archive: &Archive) -> anyhow::Result<Self> where Self: Sized;
    fn handle_event(&mut self, window: &mut Window, event: &WindowEvent);
    fn update_and_render(&mut self, delta: f64, batch: &mut Batch);
}
