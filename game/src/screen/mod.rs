mod title;
mod town;

use title::*;
use town::*;

use mpq::Archive;

use gfx::Batch;

use crate::msg::MsgBus;

#[derive(Debug, Clone, Copy)]
pub enum GameScreenName {
    Title,
    Town,
}

impl GameScreenName {
    pub fn init(&self, archive: &Archive) -> anyhow::Result<Box<dyn GameScreen>> {
        match self {
            GameScreenName::Title => Ok(Box::new(TitleScreen::new(archive)?)),
            GameScreenName::Town => Ok(Box::new(TownScreen::new(archive)?)),
        }
    }
}

/// Trait describing a "screen" of the game
/// Only one screen at a time is active, and screens take over the rendering and input handling
pub trait GameScreen {
    /// Create a new instance of this screen
    fn new(archive: &Archive) -> anyhow::Result<Self>
    where
        Self: Sized;
    /// Update the game
    fn update(&mut self, msg_bus: &mut MsgBus, delta: f64) -> Option<GameScreenName>;
    // Render the game
    fn render(&self, batch: &mut Batch);
}
