use gfx2d::{math::*, rgba, Color, Transform};

pub use soldank_shared::components::*;

#[derive(Debug)]
pub struct Sprite {
    pub group: String,
    pub name: String,
    pub sprite: Option<gfx2d::Sprite>,
    pub color: Color,
    pub transform: Transform,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            group: Default::default(),
            name: Default::default(),
            sprite: None,
            color: rgba(255, 255, 255, 255),
            transform: Transform::Pos(Vec2::ZERO),
        }
    }
}
