pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

use crate::components::Sprite;
use gfx2d::math::*;
use hecs::World;
use resources::Resources;

pub fn init(world: &mut World, resources: &mut Resources) {
    soldank_shared::physics::init(resources);

    /* Create the ground. */
    let collider = ColliderBundle {
        shape: ColliderShape::cuboid(100.0, 0.1),
        position: Vec2::new(0.0, -20.0).into(),
        ..Default::default()
    };
    world.spawn(collider);

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBundle {
        position: Vec2::new(0.0, -30.0).into(),
        ..Default::default()
    };
    let collider = ColliderBundle {
        shape: ColliderShape::ball(0.5),
        material: ColliderMaterial {
            restitution: 0.7,
            ..Default::default()
        },
        ..Default::default()
    };
    let ball = world.spawn(rigid_body);
    world.insert(ball, collider).unwrap();

    /* Ball that will be drawn */
    let sprite_scale = 0.5; // make the sprite half size than the actual PNG image
    world
        .insert_one(
            ball,
            Sprite {
                group: "Ball".into(),
                name: "Ball1".into(),
                transform: gfx2d::Transform::origin(
                    vec2(50., 50.) * (sprite_scale / -2.),
                    vec2(1.0, 1.0) * sprite_scale,
                    (0.0, vec2(50., 50.) * (sprite_scale / 2.)),
                ),
                ..Default::default()
            },
        )
        .unwrap();
}
