use super::*;
use crate::{
    engine::{input::InputState, world::WorldCameraExt},
    game,
    physics::*,
    Config, EmitterItem, Soldier,
};
use ::resources::Resources;
use std::collections::HashMap;

pub fn update_soldiers(world: &mut World, resources: &Resources, config: &Config) {
    let mut emitter = Vec::new();

    for (_entity, (mut soldier, input, rb_pos)) in world
        .query::<(&mut Soldier, Option<&Input>, Option<&RigidBodyPosition>)>()
        .iter()
    {
        if let Some(input) = input {
            soldier.control.left = input.state.contains(InputState::MoveLeft);
            soldier.control.right = input.state.contains(InputState::MoveRight);
            soldier.control.up = input.state.contains(InputState::Jump);
            soldier.control.down = input.state.contains(InputState::Crouch);
            soldier.control.fire = input.state.contains(InputState::Fire);
            soldier.control.jets = input.state.contains(InputState::Jet);
            // soldier.control.grenade = input.state.contains(InputState::);
            soldier.control.change = input.state.contains(InputState::ChangeWeapon);
            soldier.control.throw = input.state.contains(InputState::ThrowGrenade);
            soldier.control.drop = input.state.contains(InputState::DropWeapon);
            soldier.control.reload = input.state.contains(InputState::Reload);
            soldier.control.prone = input.state.contains(InputState::Prone);
            // soldier.control.flag_throw = input.state.contains(InputState::);
        }

        soldier.update(resources, &mut emitter, config);

        if let Some(rb_pos) = rb_pos {
            soldier.particle.pos = Vec2::from(rb_pos.next_position.translation) * config.phys.scale;
            soldier.particle.pos.y += 9.;
        }
    }

    for item in emitter.drain(..) {
        match item {
            EmitterItem::Bullet(_params) => {}
        };
    }
}

pub fn soldier_movement(
    world: &mut World,
    resources: &Resources,
    config: &Config,
    mouse: (f32, f32),
) {
    let mut legs_parents = HashMap::new();
    for (entity, parent) in world
        .query::<With<game::components::Legs, &Parent>>()
        .iter()
    {
        legs_parents.insert(**parent, entity);
    }

    for (body, (mut soldier, input, pawn, mut body_vel, mut body_forces, body_mp)) in world
        .query::<(
            &mut Soldier,
            &Input,
            Option<&Pawn>,
            &mut RigidBodyVelocity,
            &mut RigidBodyForces,
            &RigidBodyMassProps,
        )>()
        .iter()
    {
        if pawn.is_some() {
            let (camera, camera_position) = world.get_camera_and_camera_position();
            let (x, y) = camera.mouse_to_world(*camera_position, mouse.0, mouse.1);

            soldier.control.mouse_aim_x = x as i32;
            soldier.control.mouse_aim_y = y as i32;
        }

        if let Some(legs) = legs_parents.get(&body) {
            const RUNSPEED: f32 = 0.118;
            const RUNSPEEDUP: f32 = RUNSPEED / 6.0;
            const MAX_VELOCITY: f32 = 11.0;

            // if let Ok(contact) = world.get::<game::physics::Contact>(*legs) {
            //     println!("{:?}", *contact);
            // }

            let radius = body_vel.linvel.x.abs().clamp(3.5, 7.5) / config.phys.scale;
            if let Ok(mut shape) = world.get_mut::<ColliderShape>(*legs) {
                if let Some(ball) = shape.make_mut().as_ball_mut() {
                    ball.radius = radius;
                }

                let mut joint_set = resources.get_mut::<JointSet>().unwrap();
                for (_, joint_handle) in world.query::<&JointHandleComponent>().iter() {
                    if joint_handle.entity1() == *legs && joint_handle.entity2() == body {}
                    if let Some(joint) = joint_set.get_mut(joint_handle.handle()) {
                        joint.params = BallJoint::new(
                            Vec2::new(0.0, 0.0).into(),
                            Vec2::new(0.0, 10.5 / config.phys.scale - radius).into(),
                        )
                        .into();
                    }
                }
            }

            if let Ok(mut legs_vel) = world.get_mut::<RigidBodyVelocity>(*legs) {
                legs_vel.angvel = 0.0;

                if input.state.contains(InputState::MoveLeft)
                    && !input.state.contains(InputState::MoveRight)
                {
                    // enable only when JETting
                    // body_forces.force.x = -RUNSPEED * config.phys.scale;
                    // body_forces.force.y = -RUNSPEEDUP * config.phys.scale;
                    legs_vel.angvel = -10. * RUNSPEED * config.phys.scale;
                }
                if input.state.contains(InputState::MoveRight)
                    && !input.state.contains(InputState::MoveLeft)
                {
                    // enable only when JETting
                    // body_forces.force.x = RUNSPEED * config.phys.scale;
                    // body_forces.force.y = -RUNSPEEDUP * config.phys.scale;
                    legs_vel.angvel = 10. * RUNSPEED * config.phys.scale;
                }
                // FIXME: allow only when in contact with ground
                if input.state.contains(InputState::Jump) {
                    body_vel.apply_impulse(body_mp, Vec2::new(0.0, -RUNSPEED).into());
                    body_vel.linvel.y = f32::max(body_vel.linvel.y, -MAX_VELOCITY);
                    body_vel.linvel.y = f32::min(body_vel.linvel.y, 0.);
                }
            }
        }
    }
}
