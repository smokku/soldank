use super::*;
use gfx2d::Transform;

pub fn rotate_balls(world: &mut World, timecur: f64) {
    for (_entity, mut sprite) in world.query::<&mut Sprite>().iter() {
        if let Transform::FromOrigin { rot, .. } = &mut sprite.transform {
            rot.0 = timecur as f32 % (2. * PI);
        }
    }
}
