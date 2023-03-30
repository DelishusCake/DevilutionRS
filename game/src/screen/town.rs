use mpq::*;

use gfx::*;

use crate::msg::*;
use crate::screen::*;

#[derive(Debug)]
pub struct TownScreen {}

impl GameScreen for TownScreen {
    fn new(_archive: &Archive) -> anyhow::Result<Self> {
        Ok(Self {})
    }
    fn update(&mut self, _msg_bus: &mut MsgBus, _delta: f64) -> Option<GameScreenName> {
        None
    }
    fn render(&self, _batch: &mut Batch) {}
}
