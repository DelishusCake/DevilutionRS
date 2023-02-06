use crate::mpq::*;
use crate::gfx::*;

use crate::game::msg::*;
use crate::game::screen::*;

#[derive(Debug)]
pub struct TownScreen {
}

impl GameScreen for TownScreen {
    fn new(_archive: &Archive) -> anyhow::Result<Self> { 
        Ok(Self { })
    }
    fn update(&mut self, _msg_bus: &mut MsgBus, _delta: f64) -> Option<GameScreenName> {
        None
    }
    fn render(&self, _batch: &mut Batch) {

    }
}
