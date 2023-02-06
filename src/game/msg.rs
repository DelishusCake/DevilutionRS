use std::collections::VecDeque;

/// Game message structure
#[derive(Debug, Clone, Copy)]
pub enum Msg {
	Key(glfw::Key, glfw::Action),
}

/// In game message bus
/// Used for asynchronous communication
/// TODO: Switch to a priority queue to allow events to be delayed and re-ordered
#[derive(Debug)]
pub struct MsgBus(VecDeque<Msg>);

impl MsgBus {
	pub fn new(capacity: usize) -> Self {
		Self(VecDeque::with_capacity(capacity))
	}

	pub fn enqueue(&mut self, msg: Msg) {
		self.0.push_back(msg)
	}

	pub fn dequeue(&mut self) -> Option<Msg> {
		self.0.pop_front()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}
