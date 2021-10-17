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
            soldier.control.prone = input.state.contains(InputState::Prone);
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
