mod title;

pub use title::*;

use glfw::{Window, WindowEvent};

use crate::gfx::Batch;
use crate::mpq::Archive;

/// Trait describing a "screen" of the game
/// Only one screen at a time is active, and screens take over the rendering and input handling
pub trait GameScreen {
    /// Create a new instance of this screen
    fn new(archive: &Archive) -> anyhow::Result<Self> where Self: Sized;
    /// Handle a window event
    /// NOTE: Usually used for input handling
    fn handle_event(&mut self, window: &mut Window, event: &WindowEvent);
    /// Update and render the screen
    /// NOTE: Called within the rendering loop to update the game
    fn update_and_render(&mut self, delta: f64, batch: &mut Batch);
}

