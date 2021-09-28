use super::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Team {
    None,
    Alpha,
    Bravo,
    Charlie,
    Delta,
}

impl Default for Team {
    fn default() -> Team {
        Team::None
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EmitterItem {
    Bullet(BulletParams),
}

pub struct MainState {
    pub camera: Vec2,
    pub camera_prev: Vec2,
    pub mouse: Vec2,
    pub mouse_prev: Vec2,
    pub mouse_phys: Vec2,
    pub mouse_pressed: bool,
    pub game_width: f32,
    pub game_height: f32,
    pub zoom: f32,
    pub mouse_over_ui: bool,
}
