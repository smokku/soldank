use crate::{components::*, math::*, particles::Particle};
use gfx2d::Transform;
use hecs::World;

pub fn rotate_balls(world: &mut World, timecur: f64) {
    for (_entity, mut sprite) in world.query::<&mut Sprite>().iter() {
        if let Transform::FromOrigin { rot, .. } = &mut sprite.transform {
            rot.0 = timecur as f32 % (2. * PI);
        }
    }
}

pub fn kinetic_movement(world: &mut World) {
    for (_entity, mut body) in world.query::<&mut Particle>().iter() {
        if body.active {
            body.euler();
        }
    }
}
