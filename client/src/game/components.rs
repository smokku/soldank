use crate::engine::input::InputState;
use enumflags2::BitFlags;

pub struct Pawn;

#[derive(Default, Debug)]
pub struct Input {
    pub state: BitFlags<InputState>,
}
