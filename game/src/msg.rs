use std::cmp::Ordering;
use std::collections::BinaryHeap;

const TIME_EPS: f64 = 1e-10;

/// Game message structure
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum MsgData {
    Key(glfw::Key, glfw::Action),
}

#[derive(Debug, Clone, Copy)]
pub struct Msg {
    pub id: usize,
    pub time: f64,
    pub data: MsgData,
}

impl PartialEq for Msg {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Msg {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Msg {}

impl Ord for Msg {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if (self.time - other.time).abs() >= TIME_EPS {
            if self.time > other.time {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        } else {
            self.id.cmp(&other.id)
        }
    }
}

/// In game message bus
/// Used for asynchronous communication
/// TODO: Switch to a priority queue to allow events to be delayed and re-ordered
#[derive(Debug)]
pub struct MsgBus {
    time: f64,
    last_id: usize,
    queue: BinaryHeap<Msg>,
}

impl MsgBus {
    pub fn new(_capacity: usize) -> Self {
        Self {
            time: 0f64,
            last_id: 1usize,
            queue: BinaryHeap::new(),
        }
    }

    pub fn update(&mut self, delta: f64) {
        self.time += delta;
    }

    pub fn push(&mut self, msg: MsgData) {
        let id = self.last_id + 1;
        let time = self.time;
        let msg = Msg {
            id,
            time,
            data: msg,
        };
        self.queue.push(msg);
        self.last_id += 1;
    }

    pub fn push_delayed(&mut self, msg: MsgData, delay: f64) {
        let id = self.last_id + 1;
        let time = self.time + delay;
        let msg = Msg {
            id,
            time,
            data: msg,
        };
        self.queue.push(msg);
        self.last_id += 1;
    }

    pub fn pop(&mut self) -> Option<Msg> {
        if self.is_empty() {
            return None;
        }
        self.queue.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty() || self.time < self.queue.peek().unwrap().time
    }
}
