use super::*;
use crate::{calc::*, cvars::Config, physics::*};

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

pub struct ForceMovement;

pub fn force_movement(world: &mut World, config: &Config) {
    const RUNSPEED: f32 = 0.118;
    const RUNSPEEDUP: f32 = RUNSPEED / 6.0;
    const MAX_VELOCITY: f32 = 11.0;

    for (_, (input, mut forces, mut velocity, mass_properties, pos, rb_pos)) in world
        .query::<With<
            ForceMovement,
            (
                &Input,
                &mut RigidBodyForces,
                &mut RigidBodyVelocity,
                &RigidBodyMassProps,
                Option<&mut Position>,
                Option<&RigidBodyPosition>,
            ),
        >>()
        .iter()
    {
        if input.state.contains(InputState::MoveLeft)
            && !input.state.contains(InputState::MoveRight)
        {
            forces.force.x = -RUNSPEED * config.phys.scale;
            forces.force.y = -RUNSPEEDUP * config.phys.scale;
        }
        if input.state.contains(InputState::MoveRight)
            && !input.state.contains(InputState::MoveLeft)
        {
            forces.force.x = RUNSPEED * config.phys.scale;
            forces.force.y = -RUNSPEEDUP * config.phys.scale;
        }
        if input.state.contains(InputState::Jump) {
            velocity.apply_impulse(mass_properties, Vec2::new(0.0, -RUNSPEED).into());
            velocity.linvel.y = f32::max(velocity.linvel.y, -MAX_VELOCITY);
            velocity.linvel.y = f32::min(velocity.linvel.y, 0.);
        }
        // if input.state.contains(InputState::Crouch) {
        //     delta.y += 1.;
        // }

        // println!("{:?}", forces);

        if let Some(rb_pos) = rb_pos {
            if let Some(mut pos) = pos {
                let mut vec_pos = rb_pos.position.translation.into();
                vec_pos *= config.phys.scale;
                pos.0 = lerp(pos.0, vec_pos, 0.33);
            }
        }
    }
}
