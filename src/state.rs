use super::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Team {
    None,
    Alpha,
    Bravo,
    Charlie,
    Delta,
}

#[derive(Debug, Copy, Clone)]
pub enum EmitterItem {
    Bullet(BulletParams),
}

pub struct MainState {
    pub map: MapFile,
    pub camera: Vec2,
    pub camera_prev: Vec2,
    pub mouse: Vec2,
    pub mouse_prev: Vec2,
    pub game_width: f32,
    pub game_height: f32,
    pub gravity: f32,
    pub zoom: f32,
    pub bullets: Vec<Bullet>,
    pub mouse_over_ui: bool,
}

impl Default for Team {
    fn default() -> Team {
        Team::None
    }
}
