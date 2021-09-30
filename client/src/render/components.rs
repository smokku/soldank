use crate::constants::*;
use gfx2d::{math::*, rgba, Color, Transform};
pub use soldank_shared::components::Position;

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

impl Sprite {
    pub fn new<S: Into<String>>(group: S, name: S) -> Self {
        Sprite {
            group: group.into(),
            name: name.into(),
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Cursor(Vec2);

impl std::ops::Deref for Cursor {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Cursor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub offset: Vec2,
    pub centered: bool,
    pub zoom: f32,
    pub(crate) is_active: bool,
}
impl Default for Camera {
    fn default() -> Camera {
        Camera {
            offset: vec2(0.0, 0.0),
            centered: true,
            zoom: 0.0,
            is_active: false,
        }
    }
}

impl Camera {
    pub fn viewport(&self, position: Vec2) -> (f32, f32, f32, f32) {
        let zoom = f32::exp(self.zoom);
        let pos = position + self.offset;
        let (w, h) = (zoom * GAME_WIDTH, zoom * GAME_HEIGHT);
        let (dx, dy) = (pos.x - w / 2.0, pos.y - h / 2.0);
        (dx, dy, w, h)
    }

    pub fn mouse_to_world(&self, position: Vec2, x: f32, y: f32) -> (f32, f32) {
        let (dx, dy, _w, _h) = self.viewport(position);
        let zoom = f32::exp(self.zoom);
        (dx + x * zoom, dy + y * zoom)
    }
}
