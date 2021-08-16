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

impl MainState {
    pub fn viewport(&self, frame_percent: f32) -> (f32, f32, f32, f32) {
        let zoom = f32::exp(self.zoom);
        let cam = lerp(self.camera_prev, self.camera, frame_percent);
        let (w, h) = (zoom * constants::GAME_WIDTH, zoom * constants::GAME_HEIGHT);
        let (dx, dy) = (cam.x - w / 2.0, cam.y - h / 2.0);
        (dx, dy, w, h)
    }

    pub fn mouse_to_world(&self, frame_percent: f32, x: f32, y: f32) -> (f32, f32) {
        let (dx, dy, _w, _h) = self.viewport(frame_percent);
        let zoom = f32::exp(self.zoom);
        (dx + x * zoom, dy + y * zoom)
    }
}
