pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

use crate::{
    components::Position,
    cvars::Config,
    events::{AppEvent, AppEventsQueue},
    MapFile, PolyType,
};
use ::resources::Resources;
use hecs::World;

pub fn config_update(resources: &Resources) {
    let app_events = resources.get::<AppEventsQueue>().unwrap();
    if app_events
        .iter()
        .any(|event| matches!(event, AppEvent::CvarsChanged))
    {
        let dt = resources
            .get::<Config>()
            .unwrap()
            .net
            .orb
            .read()
            .unwrap()
            .timestep_seconds as f32;
        let mut integration_parameters = resources.get_mut::<IntegrationParameters>().unwrap();
        integration_parameters.dt = dt;
        log::debug!("IntegrationParameters updated: {}", dt);
    }
}

pub fn despawn_outliers(world: &mut World, resources: &Resources) {
    const MAX_POS: f32 = 2500.;
    let mut to_despawn = Vec::new();
    let scale = resources.get::<Config>().unwrap().phys.scale;

    for (entity, pos) in world.query::<&RigidBodyPosition>().iter() {
        let x = pos.position.translation.x * scale;
        let y = pos.position.translation.y * scale;
        if !(-MAX_POS..=MAX_POS).contains(&x) || !(-MAX_POS..=MAX_POS).contains(&y) {
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
