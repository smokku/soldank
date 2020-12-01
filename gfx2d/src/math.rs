pub use glam::{vec2, vec3, Vec2, Vec3};
pub use std::f32::consts::PI;

pub type Rad = f32;
pub type Deg = f32;

pub fn rad(angle: f32) -> Rad {
    angle
}

pub fn deg(angle: f32) -> Deg {
    angle
}

// Mat2d

// Indexed by row. Works like a 3x3 matrix where last row is always [0, 0, 1]
#[derive(Debug, Copy, Clone)]
pub struct Mat2d(pub (f32, f32, f32), pub (f32, f32, f32));

impl Mat2d {
    pub fn identity() -> Mat2d {
        Mat2d((1.0, 0.0, 0.0), (0.0, 1.0, 0.0))
    }

    pub fn translate(x: f32, y: f32) -> Mat2d {
        Mat2d((1.0, 0.0, x), (0.0, 1.0, y))
    }

    pub fn scale(x: f32, y: f32) -> Mat2d {
        Mat2d((x, 0.0, 0.0), (0.0, y, 0.0))
    }

    pub fn rotate(r: Rad) -> Mat2d {
        let (c, s) = (Rad::cos(r), Rad::sin(r));
        Mat2d((c, -s, 0.0), (s, c, 0.0))
    }

    pub fn to_3x3(&self) -> [[f32; 3]; 3] {
        [
            [(self.0).0, (self.1).0, 0.0],
            [(self.0).1, (self.1).1, 0.0],
            [(self.0).2, (self.1).2, 1.0],
        ]
    }
}

impl ::std::ops::Mul<Vec2> for Mat2d {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        vec2(
            rhs.x * (self.0).0 + rhs.y * (self.0).1 + (self.0).2,
            rhs.x * (self.1).0 + rhs.y * (self.1).1 + (self.1).2,
        )
    }
}

impl ::std::ops::Mul for Mat2d {
    type Output = Mat2d;

    fn mul(self, rhs: Mat2d) -> Mat2d {
        let a = &self;
        let b = &rhs;

        Mat2d(
            (
                ((a.0).0 * (b.0).0 + (a.0).1 * (b.1).0),
                ((a.0).0 * (b.0).1 + (a.0).1 * (b.1).1),
                ((a.0).0 * (b.0).2 + (a.0).1 * (b.1).2 + (a.0).2),
            ),
            (
                ((a.1).0 * (b.0).0 + (a.1).1 * (b.1).0),
                ((a.1).0 * (b.0).1 + (a.1).1 * (b.1).1),
                ((a.1).0 * (b.0).2 + (a.1).1 * (b.1).2 + (a.1).2),
            ),
        )
    }
}
