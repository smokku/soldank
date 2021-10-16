use super::*;
use crate::{Config, EmitterItem, Soldier};
use resources::Resources;

pub fn update_soldiers(world: &mut World, resources: &Resources, config: &Config) {
    let mut emitter = Vec::new();

    for (_entity, mut soldier) in world.query::<&mut Soldier>().iter() {
        soldier.update(resources, &mut emitter, config);
    }

    for item in emitter.drain(..) {
        match item {
            EmitterItem::Bullet(_params) => {}
        };
    }
}
