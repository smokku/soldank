pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

use crate::{
    cvars::Config,
    events::{AppEvent, AppEventsQueue},
    MapFile, PolyType,
};
use ::resources::Resources;
use hecs::World;

pub fn create_map_colliders(world: &mut World, resources: &Resources, config: &Config) {
    let map = resources.get::<MapFile>().unwrap();
    let scale = config.phys.scale;

    for polygon in map.polygons.iter() {
        match polygon.polytype {
            PolyType::NoCollide | PolyType::Background | PolyType::BackgroundTransition => continue,
            _ => {}
        }

        let vertices: Vec<Point<Real>> = polygon
            .vertices
            .iter()
            .map(|v| point![v.x / scale, v.y / scale])
            .collect();
        let mut collider = ColliderBundle {
            shape: ColliderShape::triangle(vertices[0], vertices[1], vertices[2]),
            ..Default::default()
        };
        if polygon.polytype == PolyType::Bouncy {
            collider.material.restitution = polygon.bounciness;
        }
        world.spawn(collider);
    }

    for coll in map.colliders.iter() {
        if !coll.active {
            continue;
        }

        let collider = ColliderBundle {
            shape: ColliderShape::ball(coll.diameter / scale / 2.),
            position: vector![coll.x / scale, coll.y / scale].into(),
            ..Default::default()
        };
        world.spawn(collider);
    }
}
