use crate::{
    engine::{input::InputState, Engine},
    game::components::Pawn,
    math::*,
    particles::Particle,
    render::components::*,
};
use gfx2d::Transform;
use hecs::{With, World};

pub fn rotate_balls(world: &mut World, timecur: f64) {
    for (_entity, mut sprite) in world.query::<&mut Sprite>().iter() {
        if let Transform::FromOrigin { rot, .. } = &mut sprite.transform {
            rot.0 = timecur as f32 % (2. * PI);
        }
    }
}

pub fn kinetic_movement(world: &mut World) {
    for (_entity, mut body) in world.query::<&mut Particle>().iter() {
        if body.active {
            body.euler();
        }
    }
}

pub fn primitive_movement(world: &mut World, eng: &Engine) {
    let mut delta = Vec2::ZERO;

    if eng.input.state.contains(InputState::MoveLeft) {
        delta.x -= 1.;
    }
    if eng.input.state.contains(InputState::MoveRight) {
        delta.x += 1.;
    }
    if eng.input.state.contains(InputState::Jump) {
        delta.y -= 1.;
    }
    if eng.input.state.contains(InputState::Crouch) {
        delta.y += 1.;
    }

    if delta != Vec2::ZERO {
        for (_entitty, mut pos) in world.query::<With<Pawn, &mut Position>>().iter() {
            **pos += delta;
        }
    }
}
