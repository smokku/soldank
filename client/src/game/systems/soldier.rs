use super::*;
use crate::engine::input::InputState;
use crate::{engine::world::WorldCameraExt, Config, EmitterItem, Soldier};
use resources::Resources;

pub fn update_soldiers(
    world: &mut World,
    resources: &Resources,
    config: &Config,
    mouse: (f32, f32),
) {
    let mut emitter = Vec::new();

    let (camera, camera_position) = world.get_camera_and_camera_position();
    let (x, y) = camera.mouse_to_world(*camera_position, mouse.0, mouse.1);

    for (_entity, (mut soldier, input, pos)) in world
        .query::<(&mut Soldier, Option<&Input>, Option<&mut Position>)>()
        .iter()
    {
        soldier.control.mouse_aim_x = x as i32;
        soldier.control.mouse_aim_y = y as i32;

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

        if let Some(mut pos) = pos {
            pos.x = soldier.particle.pos.x;
            pos.y = soldier.particle.pos.y;
        }
    }

    for item in emitter.drain(..) {
        match item {
            EmitterItem::Bullet(_params) => {}
        };
    }
}
