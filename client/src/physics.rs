pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

use ::resources::Resources;
use gfx2d::math::*;
use hecs::World;

pub fn init(world: &mut World, resources: &mut Resources) {
    systems::init(resources);

    /* Create the ground. */
    let collider = ColliderBundle {
        shape: ColliderShape::cuboid(100.0, 0.1),
        position: Vec2::new(0.0, -20.0).into(),
        changes: ColliderChanges::all(), // FIXME: remove after implementing change detection system
        ..Default::default()
    };
    world.spawn(collider);
}
