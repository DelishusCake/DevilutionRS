use std::ops::{Add, Mul};

/// Looping description enum
/// TODO: Remove this and switch to explicit LoopingTween and OneShotTween objects?
#[derive(Debug)]
pub enum Looping {
    Loop,
    OneShot,
}

/// Tween object
/// Used for smooth movement/animation from an initial state to a final state over a defined period of time.
#[derive(Debug)]
pub struct Tween<T> {
    // Start and end values, inclusive
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
    /// Create a new tween from `a` to `b` (inclusive) over a defined duration in seconds 
    pub fn new(a: T, b: T, duration: f64, looping: Looping) -> Self  {
        Self {
            a, 
            b,
            t: 0.0,
            duration,
            looping,
        }
    }

    /// Reset the tween to it's initial state
    pub fn reset(&mut self) {
        self.t = 0.0
    }

    /// Check if the tween is complete
    pub fn is_done(&self) -> bool {
        self.t == self.duration
    }

    /// Get the percentage completion of the tween
    /// NOTE: Always in the range [0.0, 1.0]
    pub fn percentage(&self) -> f64 {
        self.t/self.duration
    }

    /// Get the current value of the tween
    /// NOTE: Always in the range [a, b] 
    /// Where a and b are the starting and ending states
    pub fn value(&self) -> T {
        let t = self.t/self.duration;
        self.a*(t-1.0) + self.b*t
    }

    /// Update the tween with a delta time
    /// Returns the current value of the tween, which can also be retreived with `value`
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

/// A "frame" of a tweenable animation
/// Used for animated sprites that have discrete frames
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
