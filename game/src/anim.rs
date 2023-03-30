use std::ops::{Add, Mul};

/// Tweenable object trait
/// Used for smooth movement/animation from an initial state to a final state over a defined period of time.
pub trait Tween<T> {
    /// Get the percentage completion of the tween
    /// NOTE: Always in the range [0.0, 1.0]
    fn percentage(&self) -> f64;
    /// Get the current value of the tween
    /// NOTE: Always in the range [a, b]
    /// Where a and b are the starting and ending states
    fn value(&self) -> T;
    /// Update the tween with a delta time
    /// Returns the current value of the tween, which can also be retreived with `value`
    fn update(&mut self, delta: f64) -> T;
}

/// One-shot tween
/// Describes a non-looping tween
#[derive(Debug)]
pub struct OneShotTween<T> {
    // Start and end values, inclusive
    a: T,
    b: T,
    // Current time value
    t: f64,
    // Total duration of the tween
    duration: f64,
}

impl<T> OneShotTween<T> {
    /// Create a new non-looping tween from `a` to `b` (inclusive) over a defined duration in seconds
    pub fn new(a: T, b: T, duration: f64) -> Self {
        Self {
            a,
            b,
            t: 0.0,
            duration,
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
}

impl<T> Tween<T> for OneShotTween<T>
where
    T: Mul<f64, Output = T> + Add<Output = T> + Copy,
{
    fn percentage(&self) -> f64 {
        self.t / self.duration
    }

    fn value(&self) -> T {
        let t = self.percentage();
        self.a * (t - 1.0) + self.b * t
    }

    fn update(&mut self, delta: f64) -> T {
        // Increment the t value, clamping at the duration
        self.t = f64::min(self.t + delta, self.duration);
        // Return the current value
        self.value()
    }
}

/// Looping tween
/// Describes a tween that continues forever
#[derive(Debug)]
pub struct LoopingTween<T> {
    // Start and end values, inclusive
    a: T,
    b: T,
    // Current time value
    t: f64,
    // Total duration of the tween
    duration: f64,
}

impl<T> LoopingTween<T> {
    /// Create a new non-looping tween from `a` to `b` (inclusive) over a defined duration in seconds.
    /// Loops back to the begining state when complete
    pub fn new(a: T, b: T, duration: f64) -> Self {
        Self {
            a,
            b,
            t: 0.0,
            duration,
        }
    }
}

impl<T> Tween<T> for LoopingTween<T>
where
    T: Mul<f64, Output = T> + Add<Output = T> + Copy,
{
    fn percentage(&self) -> f64 {
        self.t / self.duration
    }

    fn value(&self) -> T {
        let t = self.percentage();
        self.a * (t - 1.0) + self.b * t
    }

    fn update(&mut self, delta: f64) -> T {
        // Calculate the new t value
        self.t = {
            // Increment by the delta
            let t = self.t + delta;
            // Wrap the t value around the duration
            if t >= self.duration {
                t - self.duration
            } else {
                t
            }
        };
        // Return the current value
        self.value()
    }
}

/// A "frame" of a tweenable animation
/// Used for animated sprites that have discrete frames
#[derive(Debug, Clone, Copy)]
pub struct Frame(pub usize);

impl From<usize> for Frame {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Into<usize> for Frame {
    fn into(self) -> usize {
        self.0
    }
}

impl Mul<f64> for Frame {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self((self.0 as f64 * rhs) as usize)
    }
}

impl Add<Frame> for Frame {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
