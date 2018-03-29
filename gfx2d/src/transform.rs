use super::*;

#[derive(Debug, Copy, Clone)]
pub enum Transform {
    // Simple position transform. Matrix = T(x, y).
    Pos(Vec2),

    // Rotation is done around a rotation center but position and scale are done
    // from the origin (top-left corner in the case of sprites).
    // Matrix = T(x + rot.x, y + rot.y) * R(rot) * T(-rot.x, -rot.y) * S(scale.x, scale.y).
    FromOrigin {
        pos:   Vec2,
        scale: Vec2,
        rot:   (f32, Vec2),
    },

    // Position, rotation and scale are all done relative to a pivot point.
    // Matrix = T(x, y) * R(rot) * S(scale.x, scale.y) * T(-pivot.x, -pivot.y).
    WithPivot {
        pivot: Vec2,
        pos:   Vec2,
        scale: Vec2,
        rot:   f32,
    },

    // Orthographic projection (with 'y' from top to bottom)
    Ortho {
        left:   f32,
        right:  f32,
        top:    f32,
        bottom: f32,
    },
}

impl Transform {
    pub fn none() -> Transform {
        Transform::Pos(vec2(0.0, 0.0))
    }

    pub fn pos(x: f32, y: f32) -> Transform {
        Transform::Pos(vec2(x, y))
    }

    pub fn origin(pos: Vec2, scale: Vec2, rot: (f32, Vec2)) -> Transform {
        Transform::FromOrigin{pos, scale, rot}
    }

    pub fn pivot(pivot: Vec2, pos: Vec2, scale: Vec2, rot: f32) -> Transform {
        Transform::WithPivot{pivot, pos, scale, rot}
    }

    pub fn ortho(left: f32, right: f32, top: f32, bottom: f32) -> Transform {
        Transform::Ortho{left, right, top, bottom}
    }

    pub fn matrix(&self) -> Mat2d {
        match *self {
            Transform::Pos(p) => Mat2d::translate(p.x, p.y),

            Transform::FromOrigin{pos, scale, rot} => {
                let (s, c) = (f32::sin(rot.0), f32::cos(rot.0));

                Mat2d (
                    (c*scale.x, -s*scale.y, pos.x + rot.1.y*s - c*rot.1.x + rot.1.x),
                    (s*scale.x,  c*scale.y, pos.y - rot.1.x*s - c*rot.1.y + rot.1.y),
                )
            },

            Transform::WithPivot{pivot, pos, scale, rot} => {
                let (s, c) = (f32::sin(rot), f32::cos(rot));

                let m = (
                    (c*scale.x, -s*scale.y),
                    (s*scale.x,  c*scale.y),
                );

                Mat2d (
                    ((m.0).0, (m.0).1, pos.x - pivot.y * (m.0).1 - pivot.x * (m.0).0),
                    ((m.1).0, (m.1).1, pos.y - pivot.y * (m.1).1 - pivot.x * (m.1).0),
                )
            },

            Transform::Ortho{left, right, top, bottom} => {
                let (w, h) = (right - left, top - bottom);

                Mat2d (
                    (2.0/w,  0.0 , -(right + left)/w),
                    ( 0.0 , 2.0/h, -(top + bottom)/h)
                )
            },
        }
    }
}
