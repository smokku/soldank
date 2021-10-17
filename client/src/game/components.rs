use crate::{bullet::BulletParams, engine::input::InputState};
use enumflags2::BitFlags;

pub struct Pawn;

#[derive(Default, Debug)]
pub struct Input {
    pub state: BitFlags<InputState>,
}

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
