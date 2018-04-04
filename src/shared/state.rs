use shared::calc::*;
use shared::mapfile::MapFile;
use shared::parts::ParticleSystem;

pub struct MainState {
    pub map: MapFile,
    pub soldier_parts: ParticleSystem,
    pub gostek_skeleton: ParticleSystem,
    pub camera: Vec2,
    pub camera_prev: Vec2,
    pub mouse: Vec2,
    pub mouse_prev: Vec2,
    pub game_width: f32,
    pub game_height: f32,
    pub gravity: f32,
    pub zoom: f32,
}
