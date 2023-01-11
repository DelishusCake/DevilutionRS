use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
pub struct Frame(pub usize);

impl From<usize> for Frame {
    fn from(value: usize) -> Self { Self(value) }
}

impl Into<usize> for Frame {
    fn into(self) -> usize { self.0 }
}

impl Mul<f64> for Frame {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self { Self((self.0 as f64*rhs) as usize) }
}

impl Add<Frame> for Frame {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self(self.0 + rhs.0) }
}

#[derive(Debug)]
pub enum Looping {
    Loop,
    OneShot,
}

#[derive(Debug)]
pub struct Tween<T> {
    // Start and end values
    a: T,
    b: T,
    // Current time value
    t: f64,
    // Total duration of the tween
    duration: f64,
    // Looping type
    looping: Looping,
}

impl<T> Tween<T> 
where 
    T: Mul<f64, Output=T> + Add<Output=T> + Copy
{
    pub fn new(a: T, b: T, duration: f64, looping: Looping) -> Self  {
        Self {
            a, 
            b,
            t: 0.0,
            duration,
            looping,
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0
    }

    pub fn is_done(&self) -> bool {
        self.t == self.duration
    }

    pub fn percentage(&self) -> f64 {
        self.t/self.duration
    }

    pub fn value(&self) -> T {
        let t = self.t/self.duration;
        self.a*(t-1.0) + self.b*t
    }

    pub fn update(&mut self, delta: f64) -> T {
        self.t = match self.looping {
            Looping::OneShot => f64::min(self.t + delta, self.duration),
            Looping::Loop => {
                let mut t = self.t + delta;
                if self.t + delta >= self.duration {
                    t -= self.duration; 
                }
                t
            },
        };
        self.value() 
    }
}
