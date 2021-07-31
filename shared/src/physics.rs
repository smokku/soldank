use crate::components::Parent;
use hecs::{Bundle, Entity, Query, Without, World};
use rapier2d::{
    data::{ComponentSet, ComponentSetMut, ComponentSetOption, Index},
    prelude::*,
};
use resources::Resources;
use std::ops::Deref;

pub trait IntoHandle<H> {
    fn handle(self) -> H;
}

pub trait IntoEntity {
    fn entity(self) -> Entity;
}

impl IntoHandle<Index> for Entity {
    #[inline]
    fn handle(self) -> Index {
        let bits = self.to_bits();
        Index::from_raw_parts(bits as u32, (bits >> 32) as u32)
    }
}

impl IntoEntity for Index {
    #[inline]
    fn entity(self) -> Entity {
        let (id, gen) = self.into_raw_parts();
        let bits = u64::from(gen) << 32 | u64::from(id);
        Entity::from_bits(bits)
    }
}

impl IntoHandle<RigidBodyHandle> for Entity {
    #[inline]
    fn handle(self) -> RigidBodyHandle {
        let bits = self.to_bits();
        RigidBodyHandle::from_raw_parts(bits as u32, (bits >> 32) as u32)
    }
}

impl IntoEntity for RigidBodyHandle {
    #[inline]
    fn entity(self) -> Entity {
        self.0.entity()
    }
}

impl IntoHandle<ColliderHandle> for Entity {
    #[inline]
    fn handle(self) -> ColliderHandle {
        let bits = self.to_bits();
        ColliderHandle::from_raw_parts(bits as u32, (bits >> 32) as u32)
    }
}

impl IntoEntity for ColliderHandle {
    #[inline]
    fn entity(self) -> Entity {
        self.0.entity()
    }
}

struct AddedColliderParent; // FIXME: remove after adding change tracking to hecs

pub fn init(resources: &mut Resources) {
    resources.insert(PhysicsPipeline::new());
    resources.insert(IslandManager::new());
    resources.insert(BroadPhase::new());
    resources.insert(NarrowPhase::new());
    resources.insert(JointSet::new());
    resources.insert(CCDSolver::new());
}

pub fn step(world: &mut World, resources: &Resources, dt: f32) {
    // println!("step");
    let gravity = vector![0.0, 9.81];
    let integration_parameters = IntegrationParameters {
        dt,
        ..Default::default()
    };
    let physics_hooks = ();
    let event_handler = ();

    let mut physics_pipeline = resources.get_mut::<PhysicsPipeline>().unwrap();
    let mut island_manager = resources.get_mut::<IslandManager>().unwrap();
    let mut broad_phase = resources.get_mut::<BroadPhase>().unwrap();
    let mut narrow_phase = resources.get_mut::<NarrowPhase>().unwrap();
    let mut joint_set = resources.get_mut::<JointSet>().unwrap();
    let mut ccd_solver = resources.get_mut::<CCDSolver>().unwrap();

    // ----------- attach_bodies_and_colliders_system --------
    {
        // println!("attach_bodies_and_colliders_system");
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

    // ----------- finalize_collider_attach_to_bodies --------
    {
        // println!("finalize_collider_attach_to_bodies");
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

                // modif_tracker
                //     .body_colliders
                //     .entry(co_parent.handle)
                //     .or_insert(vec![])
                //     .push(collider_entity.handle());
                // modif_tracker
                //     .colliders_parent
                //     .insert(collider_entity.handle(), co_parent.handle);

                *co_changes = ColliderChanges::default();
                *co_bf_data = ColliderBroadPhaseData::default();
                rb_colliders.attach_collider(
                    &mut rb_changes,
                    &mut rb_ccd,
                    &mut rb_mprops,
                    &rb_pos,
                    collider_entity.handle(),
                    &mut co_pos,
                    &co_parent,
                    &co_shape,
                    &co_mprops,
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

macro_rules! impl_component_set_option(
    ($ComponentsSet: ident, $T: ty) => {
        impl<'a> ComponentSetOption<$T> for $ComponentsSet<'a> {
            fn get(&self, handle: Index) -> Option<&$T> {
                let ret = self.0
                    .get::<$T>(handle.entity())
                    .ok()
                    .map(|data| unsafe {
                        let data = data.deref() as *const $T;
                        &*data
                    });
                // println!("ComponentSetOption get {:?} {} : {:?}", handle, std::any::type_name::<$T>(), ret);
                ret
            }
        }
    }
);
macro_rules! impl_component_set(
    ($ComponentsSet: ident, $T: ty) => {
        impl<'a> ComponentSet<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn size_hint(&self) -> usize {
                // println!("ComponentSet size_hint");
                0
            }

            #[inline(always)]
            fn for_each(&self, mut f: impl FnMut(Index, &$T)) {
                // println!("ComponentSet for_each");
                self.0
                    .query::<&$T>()
                    .iter()
                    .for_each(|(entity, data)| f(entity.handle(), data));
            }
        }
    }
);
macro_rules! impl_component_set_mut(
    ($ComponentsSet: ident, $T: ty) => {
        impl<'a> ComponentSetMut<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn set_internal(&mut self, handle: Index, val: $T) {
                // println!("ComponentSetMut set_internal {:?} {} : {:?}", handle, std::any::type_name::<$T>(), val);
                let _ = self.0
                    .get_mut::<$T>(handle.entity())
                    .map(|mut data| *data = val);
            }

            #[inline(always)]
            fn map_mut_internal<Result>(
                &mut self,
                handle: Index,
                f: impl FnOnce(&mut $T) -> Result,
            ) -> Option<Result> {
                // println!("ComponentSetMut map_mut_internal");
                self.0
                    .get_mut::<$T>(handle.entity())
                    .map(|mut data| f(&mut data))
                    .ok()
            }
        }
    }
);

pub struct RigidBodyComponentsSet<'a>(&'a World);

// impl<'a> ComponentSetOption<RigidBodyPosition> for RigidBodyComponentsSet<'a> {
//     fn get(&self, handle: Index) -> Option<&RigidBodyPosition> {
//         self.0
//             .get::<RigidBodyPosition>(handle.entity())
//             .ok()
//             .map(|data| unsafe {
//                 let data = data.deref() as *const RigidBodyPosition;
//                 &*data
//             })
//     }
// }
// impl<'a> ComponentSet<RigidBodyPosition> for RigidBodyComponentsSet<'a> {
//     #[inline(always)]
//     fn size_hint(&self) -> usize {
//         0
//     }

//     #[inline(always)]
//     fn for_each(&self, mut f: impl FnMut(Index, &RigidBodyPosition)) {
//         self.0
//             .query::<&RigidBodyPosition>()
//             .iter()
//             .for_each(|(entity, data)| f(entity.handle(), data));
//     }
// }
// impl<'a> ComponentSetMut<RigidBodyPosition> for RigidBodyComponentsSet<'a> {
//     #[inline(always)]
//     fn set_internal(&mut self, handle: Index, val: RigidBodyPosition) {
//         self.0
//             .get_mut::<RigidBodyPosition>(handle.entity())
//             .map(|mut data| *data = val);
//     }

//     #[inline(always)]
//     fn map_mut_internal<Result>(
//         &mut self,
//         handle: Index,
//         f: impl FnOnce(&mut RigidBodyPosition) -> Result,
//     ) -> Option<Result> {
//         self.0
//             .get_mut::<RigidBodyPosition>(handle.entity())
//             .map(|mut data| f(&mut data))
//             .ok()
//     }
// }

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyPosition);
impl_component_set!(RigidBodyComponentsSet, RigidBodyPosition);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyPosition);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyVelocity);
impl_component_set!(RigidBodyComponentsSet, RigidBodyVelocity);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyVelocity);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyMassProps);
impl_component_set!(RigidBodyComponentsSet, RigidBodyMassProps);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyMassProps);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyIds);
impl_component_set!(RigidBodyComponentsSet, RigidBodyIds);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyIds);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyForces);
impl_component_set!(RigidBodyComponentsSet, RigidBodyForces);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyForces);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyActivation);
impl_component_set!(RigidBodyComponentsSet, RigidBodyActivation);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyActivation);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyChanges);
impl_component_set!(RigidBodyComponentsSet, RigidBodyChanges);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyChanges);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyCcd);
impl_component_set!(RigidBodyComponentsSet, RigidBodyCcd);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyCcd);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyColliders);
impl_component_set!(RigidBodyComponentsSet, RigidBodyColliders);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyColliders);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyDamping);
impl_component_set!(RigidBodyComponentsSet, RigidBodyDamping);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyDominance);
impl_component_set!(RigidBodyComponentsSet, RigidBodyDominance);

impl_component_set_option!(RigidBodyComponentsSet, RigidBodyType);
impl_component_set!(RigidBodyComponentsSet, RigidBodyType);

#[derive(Bundle)]
pub struct RigidBodyBundle {
    pub body_type: RigidBodyType,
    pub position: RigidBodyPosition,
    pub velocity: RigidBodyVelocity,
    pub mass_properties: RigidBodyMassProps,
    pub forces: RigidBodyForces,
    pub activation: RigidBodyActivation,
    pub damping: RigidBodyDamping,
    pub dominance: RigidBodyDominance,
    pub ccd: RigidBodyCcd,
    pub changes: RigidBodyChanges,
    pub ids: RigidBodyIds,
    pub colliders: RigidBodyColliders,
}

impl Default for RigidBodyBundle {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            position: RigidBodyPosition::default(),
            velocity: RigidBodyVelocity::default(),
            mass_properties: RigidBodyMassProps::default(),
            forces: RigidBodyForces::default(),
            activation: RigidBodyActivation::default(),
            damping: RigidBodyDamping::default(),
            dominance: RigidBodyDominance::default(),
            ccd: RigidBodyCcd::default(),
            changes: RigidBodyChanges::default(),
            ids: RigidBodyIds::default(),
            colliders: RigidBodyColliders::default(),
        }
    }
}

pub struct ColliderComponentsSet<'a>(&'a World);

impl_component_set_option!(ColliderComponentsSet, ColliderChanges);
impl_component_set!(ColliderComponentsSet, ColliderChanges);
impl_component_set_mut!(ColliderComponentsSet, ColliderChanges);

impl_component_set_option!(ColliderComponentsSet, ColliderPosition);
impl_component_set!(ColliderComponentsSet, ColliderPosition);
impl_component_set_mut!(ColliderComponentsSet, ColliderPosition);

impl_component_set_option!(ColliderComponentsSet, ColliderBroadPhaseData);
impl_component_set!(ColliderComponentsSet, ColliderBroadPhaseData);
impl_component_set_mut!(ColliderComponentsSet, ColliderBroadPhaseData);

impl_component_set_option!(ColliderComponentsSet, ColliderShape);
impl_component_set!(ColliderComponentsSet, ColliderShape);

impl_component_set_option!(ColliderComponentsSet, ColliderType);
impl_component_set!(ColliderComponentsSet, ColliderType);

impl_component_set_option!(ColliderComponentsSet, ColliderMaterial);
impl_component_set!(ColliderComponentsSet, ColliderMaterial);

impl_component_set_option!(ColliderComponentsSet, ColliderFlags);
impl_component_set!(ColliderComponentsSet, ColliderFlags);

impl_component_set_option!(ColliderComponentsSet, ColliderParent);

#[derive(Query, Debug, PartialEq)]
pub struct RigidBodyComponentsQuery<'a> {
    pub position: &'a RigidBodyPosition,
    pub velocity: &'a RigidBodyVelocity,
    pub mass_props: &'a RigidBodyMassProps,
    pub ids: &'a RigidBodyIds,
    pub forces: &'a RigidBodyForces,
    pub activation: &'a RigidBodyActivation,
    pub changes: &'a RigidBodyChanges,
    pub ccd: &'a RigidBodyCcd,
    pub coliders: &'a RigidBodyColliders,
    pub damping: &'a RigidBodyDamping,
    pub dominance: &'a RigidBodyDominance,
    pub typ: &'a RigidBodyType,
}

#[derive(Query)]
pub struct ColliderComponentsQuery<'a> {
    pub changes: &'a ColliderChanges,
    pub position: &'a ColliderPosition,
    pub broad_phase_data: &'a ColliderBroadPhaseData,
    pub shape: &'a ColliderShape,
    pub typ: &'a ColliderType,
    pub material: &'a ColliderMaterial,
    pub flags: &'a ColliderFlags,
    pub parent: Option<&'a ColliderParent>,
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider_type: ColliderType,
    pub shape: ColliderShape,
    pub position: ColliderPosition,
    pub material: ColliderMaterial,
    pub flags: ColliderFlags,
    pub mass_properties: ColliderMassProps,
    pub changes: ColliderChanges,
    pub bf_data: ColliderBroadPhaseData,
}

impl Default for ColliderBundle {
    fn default() -> Self {
        Self {
            collider_type: ColliderType::Solid,
            shape: ColliderShape::ball(0.5),
            position: ColliderPosition::default(),
            material: ColliderMaterial::default(),
            flags: ColliderFlags::default(),
            mass_properties: ColliderMassProps::default(),
            changes: ColliderChanges::default(),
            bf_data: ColliderBroadPhaseData::default(),
        }
    }
}
