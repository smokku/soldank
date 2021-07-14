use crate::{components, constants::*};
use hecs::World;
use rapier2d::prelude::*;
use resources::Resources;

pub fn init(resources: &mut Resources) {
    resources.insert(PhysicsPipeline::new());
    resources.insert(IslandManager::new());
    resources.insert(BroadPhase::new());
    resources.insert(NarrowPhase::new());
    resources.insert(RigidBodySet::new());
    resources.insert(ColliderSet::new());
    resources.insert(JointSet::new());
    resources.insert(CCDSolver::new());
}

pub fn step(resources: &Resources) {
    let gravity = vector![0.0, 9.81];
    let integration_parameters = IntegrationParameters {
        dt: TIMESTEP_RATE as f32,
        ..Default::default()
    };
    let physics_hooks = ();
    let event_handler = ();

    let mut physics_pipeline = resources.get_mut::<PhysicsPipeline>().unwrap();
    let mut island_manager = resources.get_mut::<IslandManager>().unwrap();
    let mut broad_phase = resources.get_mut::<BroadPhase>().unwrap();
    let mut narrow_phase = resources.get_mut::<NarrowPhase>().unwrap();
    let mut rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
    let mut collider_set = resources.get_mut::<ColliderSet>().unwrap();
    let mut joint_set = resources.get_mut::<JointSet>().unwrap();
    let mut ccd_solver = resources.get_mut::<CCDSolver>().unwrap();

    physics_pipeline.step(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut rigid_body_set,
        &mut collider_set,
        &mut joint_set,
        &mut ccd_solver,
        &physics_hooks,
        &event_handler,
    );
}

pub fn sync_to_world(world: &mut World, resources: &Resources, scale: f32) {
    let rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();

    for (_entity, (body_handle, position)) in world
        .query::<(&RigidBodyHandle, &mut components::Position)>()
        .iter()
    {
        let ball_body = &rigid_body_set[*body_handle];
        position.x = ball_body.translation().x * scale;
        position.y = ball_body.translation().y * scale;
    }
}
