use super::*;
use crate::{components::Parent, physics::resources::ModificationTracker};
use ::resources::Resources;
use hecs::{Added, Without, World};

pub fn init(resources: &mut Resources) {
    resources.insert(PhysicsPipeline::new());
    // resources.insert(QueryPipeline::new());
    // resources.insert(RapierConfiguration::default());
    resources.insert(IntegrationParameters::default());
    resources.insert(BroadPhase::new());
    resources.insert(NarrowPhase::new());
    resources.insert(IslandManager::new());
    resources.insert(JointSet::new());
    resources.insert(CCDSolver::new());
    // resources.insert(Events::<IntersectionEvent>::default());
    // resources.insert(Events::<ContactEvent>::default());
    // resources.insert(SimulationToRenderTime::default());
    // resources.insert(JointsEntityMap::default());
    resources.insert(ModificationTracker::default());
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

/// System responsible for performing one timestep of the physics world.
pub fn step_world(world: &mut World, resources: &Resources) {
    // println!("step");
    use std::mem::replace;

    let gravity = vector![0.0, 9.81];

    // let configuration = resources.get::<RapierConfiguration>().unwrap();
    let integration_parameters = resources.get::<IntegrationParameters>().unwrap();
    let mut modifs_tracker = resources.get_mut::<ModificationTracker>().unwrap();

    let mut physics_pipeline = resources.get_mut::<PhysicsPipeline>().unwrap();
    // let mut query_pipeline = resources.get_mut::<QueryPipeline>().unwrap();
    let mut island_manager = resources.get_mut::<IslandManager>().unwrap();
    let mut broad_phase = resources.get_mut::<BroadPhase>().unwrap();
    let mut narrow_phase = resources.get_mut::<NarrowPhase>().unwrap();
    let mut ccd_solver = resources.get_mut::<CCDSolver>().unwrap();
    let mut joint_set = resources.get_mut::<JointSet>().unwrap();
    // let mut joints_entity_map = resources.get_mut::<JointsEntityMap>().unwrap();
    let physics_hooks = ();
    let event_handler = ();

    modifs_tracker.detect_removals(world);
    modifs_tracker.detect_modifications(world);

    let mut rigid_body_components_set = RigidBodyComponentsSet(world);
    let mut collider_components_set = ColliderComponentsSet(world);

    let cleanup_entities = modifs_tracker.propagate_removals(
        &mut island_manager,
        &mut rigid_body_components_set,
        // &mut joints,
        // &mut joints_entity_map,
    );
    island_manager.cleanup_removed_rigid_bodies(&mut rigid_body_components_set);

    physics_pipeline.step_generic(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut rigid_body_components_set,
        &mut collider_components_set,
        &mut replace(&mut modifs_tracker.modified_bodies, vec![]),
        &mut replace(&mut modifs_tracker.modified_colliders, vec![]),
        &mut replace(&mut modifs_tracker.removed_colliders, vec![]),
        &mut joint_set,
        &mut ccd_solver,
        &physics_hooks,
        &event_handler,
    );

    for entity in cleanup_entities {
        let _ = world.remove::<ColliderBundle>(entity);
        let _ = world.remove_one::<ColliderParent>(entity);
    }
    modifs_tracker.clear_modified_and_removed();
}

/// System responsible for creating a Rapier rigid-body and collider from their
/// builder resources.
pub fn attach_bodies_and_colliders(world: &mut World) {
    // println!("attach_bodies_and_colliders");
    let mut co_parents = Vec::new();
    'outer: for (collider_entity, co_pos) in world
        .query::<Without<
            ColliderParent,
            // Colliders.
            &ColliderPosition,
        >>()
        .iter()
    {
        // Find the closest ancestor (possibly the same entity) with a body
        let mut body_entity = collider_entity;
        loop {
            if world.get::<RigidBodyPosition>(body_entity).is_ok() {
                // Found it!
                break;
            } else if let Ok(parent_entity) = world.get::<Parent>(body_entity) {
                body_entity = **parent_entity;
            } else {
                continue 'outer;
            }
        }

        let co_parent = ColliderParent {
            pos_wrt_parent: co_pos.0,
            handle: body_entity.handle(),
        };
        co_parents.push((collider_entity, co_parent));
    }
    for (collider_entity, co_parent) in co_parents.drain(..) {
        world.insert_one(collider_entity, co_parent).unwrap();
    }
}

/// System responsible for creating a Rapier rigid-body and collider from their
/// builder resources.
pub fn finalize_collider_attach_to_bodies(world: &mut World, resources: &Resources) {
    // println!("finalize_collider_attach_to_bodies");
    let mut modif_tracker = resources.get_mut::<ModificationTracker>().unwrap();

    for (
        collider_entity,
        (
            mut co_changes,
            mut co_bf_data,
            mut co_pos,
            co_shape,
            co_mprops,
            co_parent,
            _added_colider_parent,
        ),
    ) in world
        .query::<(
            // Collider.
            &mut ColliderChanges,
            &mut ColliderBroadPhaseData,
            &mut ColliderPosition,
            &ColliderShape,
            &ColliderMassProps,
            &ColliderParent,
            Added<ColliderParent>,
        )>()
        .iter()
        .filter(|(_e, (_, _, _, _, _, _, added))| *added)
    {
        let mut body_query = world.query_one::<(
                // Rigid-bodies.
                &mut RigidBodyChanges,
                &mut RigidBodyCcd,
                &mut RigidBodyColliders,
                &mut RigidBodyMassProps,
                &RigidBodyPosition,
            )>(co_parent.handle.entity()).unwrap();
        if let Some((mut rb_changes, mut rb_ccd, mut rb_colliders, mut rb_mprops, rb_pos)) =
            body_query.get()
        {
            // Contract:
            // - Reset collider's references.
            // - Set collider's parent handle.
            // - Attach the collider to the body.

            // Update the modification tracker.
            // NOTE: this must be done before the `.attach_collider` because
            //       `.attach_collider` will set the `MODIFIED` flag.

            if !rb_changes.contains(RigidBodyChanges::MODIFIED) {
                modif_tracker.modified_bodies.push(co_parent.handle);
            }

            modif_tracker
                .body_colliders
                .entry(co_parent.handle)
                .or_insert_with(Vec::new)
                .push(collider_entity.handle());
            modif_tracker
                .colliders_parent
                .insert(collider_entity.handle(), co_parent.handle);

            *co_changes = ColliderChanges::default();
            *co_bf_data = ColliderBroadPhaseData::default();
            rb_colliders.attach_collider(
                &mut rb_changes,
                &mut rb_ccd,
                &mut rb_mprops,
                rb_pos,
                collider_entity.handle(),
                &mut co_pos,
                co_parent,
                co_shape,
                co_mprops,
            );
        }
    }
}

/// System responsible for collecting the entities with removed rigid-bodies, colliders,
/// or joints.
pub fn collect_removals(world: &mut World, resources: &Resources) {
    let mut modification_tracker = resources.get_mut::<ModificationTracker>().unwrap();

    modification_tracker.detect_removals(world);
}
