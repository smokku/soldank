use crate::math::{vec2, Vec2};
use derive_deref::{Deref, DerefMut};
use nanoserde::{DeBin, DeBinErr, SerBin};

#[derive(Debug, Clone, DeBin, SerBin)]
pub struct Nick(pub String);

#[derive(Debug, Clone, DeBin, SerBin)]
pub struct Soldier;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Position(Vec2);

impl Position {
    pub fn new<P: Into<f32>>(x: P, y: P) -> Self {
        Position(vec2(x.into(), y.into()))
    }
}

#[derive(Debug, Clone, DeBin, SerBin)]
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

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub active: bool,
    pub pos: Vec2,
    pub old_pos: Vec2,
    pub velocity: Vec2,
    pub force: Vec2,
    pub one_over_mass: f32,
    pub timestep: f32,
    pub gravity: f32,
    pub e_damping: f32,
    pub v_damping: f32,
}

impl Default for Particle {
    fn default() -> Particle {
        Particle {
            active: false,
            pos: Vec2::ZERO,
            old_pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
            one_over_mass: 0.0,
            timestep: 0.0,
            gravity: 0.0,
            e_damping: 0.0,
            v_damping: 0.0,
        }
    }
}

impl Particle {
    pub fn euler(&mut self) {
        self.old_pos = self.pos;
        self.force.y += self.gravity;
        self.velocity += self.force * self.one_over_mass * self.timestep.powi(2);
        self.pos += self.velocity;
        self.velocity *= self.e_damping;
        self.force = Vec2::ZERO;
    }

    pub fn verlet(&mut self) {
        let a = self.pos * (1.0 + self.v_damping);
        let b = self.old_pos * self.v_damping;

        self.old_pos = self.pos;
        self.force.y += self.gravity;
        self.pos = a - b + self.force * self.one_over_mass * self.timestep.powi(2);
        self.force = Vec2::ZERO;
    }
}
