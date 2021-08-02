use crate::components;
use gfx2d::{math::*, Transform};
use hecs::World;

pub fn rotate_balls(world: &mut World, timecur: f64) {
    for (_entity, mut sprite) in world.query::<&mut components::Sprite>().iter() {
        if let Transform::FromOrigin { rot, .. } = &mut sprite.transform {
            rot.0 = timecur as f32 % (2. * PI);
        }
    }
}
