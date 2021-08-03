use super::*;
use crate::components::Parent;
use ::resources::Resources;
use hecs::{Without, World};

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
    resources.insert(resources::ModificationTracker::default());
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

pub fn step_world(world: &mut World, resources: &Resources) {
    // println!("step");
    let gravity = vector![0.0, 9.81];

    // let configuration = resources.get::<RapierConfiguration>().unwrap();
    let integration_parameters = resources.get::<IntegrationParameters>().unwrap();

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

    let mut rigid_body_components_set = RigidBodyComponentsSet(world);
    let mut collider_components_set = ColliderComponentsSet(world);

    // let mut modified_bodies = Vec::new(); // FIXME: implement
    // let mut modified_colliders = Vec::new(); // FIXME: implement
    let mut removed_colliders = Vec::new(); // FIXME: implement

    let mut modified_bodies = world
        .query::<RigidBodyComponentsQuery>()
        .iter()
        .map(|(entity, _rb)| RigidBodyHandle(entity.handle()))
        .collect();
    let mut modified_colliders = world
        .query::<ColliderComponentsQuery>()
        .iter()
        .map(|(entity, _co)| ColliderHandle(entity.handle()))
        .collect();

    physics_pipeline.step_generic(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut rigid_body_components_set,
        &mut collider_components_set,
        &mut modified_bodies,
        &mut modified_colliders,
        &mut removed_colliders,
        &mut joint_set,
        &mut ccd_solver,
        &physics_hooks,
        &event_handler,
    );
}

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
        world
            .insert_one(collider_entity, AddedColliderParent)
            .unwrap();
    }
}

pub fn finalize_collider_attach_to_bodies(world: &mut World, resources: &Resources) {
    // println!("finalize_collider_attach_to_bodies");
    let mut modif_tracker = resources
        .get_mut::<resources::ModificationTracker>()
        .unwrap();

    let mut remove_added_collider_parent = Vec::new();
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
            &AddedColliderParent, // FIXME:: Added<ColliderParent>,
        )>()
        .iter()
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

            // if !rb_changes.contains(RigidBodyChanges::MODIFIED) {
            //     modif_tracker.modified_bodies.push(co_parent.handle);
            // }

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
            remove_added_collider_parent.push(collider_entity);
        }
    }
    for collider_entity in remove_added_collider_parent.drain(..) {
        world
            .remove_one::<AddedColliderParent>(collider_entity)
            .unwrap();
    }
}
