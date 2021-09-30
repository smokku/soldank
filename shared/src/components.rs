use crate::math::{vec2, Vec2};
use derive_deref::{Deref, DerefMut};
use nanoserde::{DeBin, DeBinErr, SerBin};

#[derive(Debug, Clone, DeBin, SerBin, Deref, DerefMut)]
pub struct Nick(pub String);

#[derive(Debug, Clone, DeBin, SerBin)]
pub struct Soldier;

#[derive(Default, Debug, Copy, Clone, Deref, DerefMut)]
pub struct Position(pub Vec2);

impl Position {
    pub fn new<P: Into<f32>>(x: P, y: P) -> Self {
        Position(vec2(x.into(), y.into()))
    }
}

#[derive(Debug, Copy, Clone, DeBin, SerBin)]
struct PositionTuple(f32, f32);

impl SerBin for Position {
    fn ser_bin(&self, output: &mut Vec<u8>) {
        let val = PositionTuple(self.0.x, self.0.y);
        val.ser_bin(output);
    }
}

impl DeBin for Position {
    fn de_bin(offset: &mut usize, bytes: &[u8]) -> Result<Self, DeBinErr> {
        let val = PositionTuple::de_bin(offset, bytes)?;
        Ok(Position::new(val.0, val.1))
    }
}
