pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

use crate::{components::Position, cvars::Config};
use ::resources::Resources;
use gfx2d::math::*;
use hecs::World;

pub fn init(world: &mut World, resources: &mut Resources) {
    systems::init(resources);

    /* Create the ground. */
    let collider = ColliderBundle {
        shape: ColliderShape::cuboid(100.0, 0.1),
        position: Vec2::new(0.0, -20.0).into(),
        ..Default::default()
    };
    world.spawn(collider);
}

pub fn despawn_outliers(world: &mut World, resources: &Resources) {
    const MAX_POS: f32 = 2500.;
    let mut to_despawn = Vec::new();
    let scale = resources.get::<Config>().unwrap().phys.scale;

    for (entity, pos) in world.query::<&RigidBodyPosition>().iter() {
        let x = pos.position.translation.x * scale;
        let y = pos.position.translation.y * scale;
        if x > MAX_POS || x < -MAX_POS || y > MAX_POS || y < -MAX_POS {
            to_despawn.push(entity);
        }
    }

    for (entity, pos) in world.query::<&Position>().iter() {
        if pos.x > MAX_POS || pos.x < -MAX_POS || pos.y > MAX_POS || pos.y < -MAX_POS {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        world.despawn(entity).unwrap();
    }
}
