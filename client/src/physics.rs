pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

use crate::{components::Position, cvars::Config, MapFile, PolyType};
use ::resources::Resources;
use hecs::World;

pub fn init(_world: &mut World, resources: &mut Resources) {
    systems::init(resources);

    // possibly spawn stuff here
}

// TODO: connect to event bus
pub fn config_update(resources: &mut Resources, dt: f32) {
    // let dt = resources
    //     .get::<Config>()
    //     .unwrap()
    //     .net
    //     .orb
    //     .read()
    //     .unwrap()
    //     .timestep_seconds as f32;
    let mut integration_parameters = resources.get_mut::<IntegrationParameters>().unwrap();
    integration_parameters.dt = dt;
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

pub fn create_map_colliders(world: &mut World, resources: &Resources) {
    let map = resources.get::<MapFile>().unwrap();
    let scale = resources.get::<Config>().unwrap().phys.scale;

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
}
