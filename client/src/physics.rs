pub use soldank_shared::physics::{step, sync_to_world};

use crate::components;
use gfx2d::math::*;
use hecs::World;
use rapier2d::prelude::*;
use resources::Resources;

pub fn init(world: &mut World, resources: &mut Resources) {
    soldank_shared::physics::init(resources);

    let mut rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
    let mut collider_set = resources.get_mut::<ColliderSet>().unwrap();

    /* Create the ground. */
    let collider = ColliderBuilder::cuboid(100.0, 0.1)
        .translation(vector![0.0, -20.0])
        .build();
    collider_set.insert(collider);

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBuilder::new_dynamic()
        .translation(vector![0.0, -30.0])
        .build();
    let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
    let ball_body_handle = rigid_body_set.insert(rigid_body);
    collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

    /* Ball entity that will be drawn */
    let sprite_scale = 0.5;
    world.spawn((
        components::Position::new(0.0, 0.0),
        components::Sprite {
            group: "Ball".into(),
            name: "Ball1".into(),
            transform: gfx2d::Transform::origin(
                vec2(50., 50.) * (sprite_scale / -2.),
                vec2(1.0, 1.0) * sprite_scale,
                (0.0, vec2(50., 50.) * (sprite_scale / 2.)),
            ),
            ..Default::default()
        },
        ball_body_handle,
    ));
}
