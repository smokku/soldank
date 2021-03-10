use legion::{system, world::SubWorld, Query};

use crate::components::Particle;

#[system]
pub fn kinetic_movement(world: &mut SubWorld, query: &mut Query<&mut Particle>) {
    for body in query.iter_mut(world) {
        if body.active {
            body.euler();
        }
    }
}
