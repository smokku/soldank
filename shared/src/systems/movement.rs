use hecs::World;

use crate::components::*;

pub fn kinetic_movement(world: &mut World) {
    for (_entity, body) in world.query::<&mut Particle>().iter() {
        if body.active {
            body.euler();
        }
    }
}
