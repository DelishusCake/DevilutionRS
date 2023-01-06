mod gpu;
mod util;
mod batch;
mod image;
mod material;

pub use gpu::*;
pub use batch::*;
pub use image::*;
pub use material::*;

use cgmath::*;

#[derive(Copy, Clone, Debug)]
pub struct Xform2D {
    rot: Basis2<f32>,
    pos: Vector2<f32>,
}

impl Xform2D {
    pub fn new(pos: Vector2<f32>, rot: Basis2<f32>) -> Self {
        Self { rot, pos }
    }

    pub fn position(pos: Vector2<f32>) -> Self {
    	Self {
    		rot: Rotation2::from_angle(Rad(0.0)),
    		pos,
    	}
    }

    pub fn identity() -> Self {
        Self {
            rot: Rotation2::from_angle(Rad(0.0f32)),
            pos: vec2(0.0f32, 0.0f32),
        }
    }

    pub fn apply(&self, v: Vector2<f32>) -> Vector2<f32> {
        self.rot.rotate_vector(v) + self.pos
    }
}
