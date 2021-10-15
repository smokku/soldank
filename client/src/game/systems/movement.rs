use super::*;

pub fn kinetic_movement(world: &mut World) {
    for (_entity, mut body) in world.query::<&mut Particle>().iter() {
        if body.active {
            body.euler();
        }
    }
}

pub struct PrimitiveMovement;

pub fn primitive_movement(world: &mut World) {
    for (_, (input, mut pos)) in world
        .query::<With<PrimitiveMovement, (&Input, &mut Position)>>()
        .iter()
    {
        let mut delta = Vec2::ZERO;

        if input.state.contains(InputState::MoveLeft) {
            delta.x -= 1.;
        }
        if input.state.contains(InputState::MoveRight) {
            delta.x += 1.;
        }
        if input.state.contains(InputState::Jump) {
            delta.y -= 1.;
        }
        if input.state.contains(InputState::Crouch) {
            delta.y += 1.;
        }

        if delta != Vec2::ZERO {
            **pos += delta;
        }
    }
}
