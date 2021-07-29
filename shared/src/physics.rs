use crate::{components, constants::*};
use hecs::{Bundle, Entity, Query, QueryBorrow, World};
use rapier2d::{
    data::{ComponentSet, ComponentSetMut, ComponentSetOption, Index},
    prelude::*,
};
use resources::Resources;

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

pub fn step(world: &World, resources: &Resources) {
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
    let mut joint_set = resources.get_mut::<JointSet>().unwrap();
    let mut ccd_solver = resources.get_mut::<CCDSolver>().unwrap();

    // let bodies_query = world.query::<RigidBodyComponentsQuery>();
    // let colliders_query = world.query::<ColliderComponentsQuery>();

    let mut rigid_body_components_set = RigidBodyComponentsSet(world);
    let mut collider_components_set = ColliderComponentsSet(world);

    let mut modified_bodies = Vec::new();
    let mut modified_colliders = Vec::new();
    let mut removed_colliders = Vec::new();

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

macro_rules! impl_component_set_mut(
    ($ComponentsSet: ident, $T: ty) => {
        impl<'a> ComponentSetOption<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn get(&self, handle: Index) -> Option<&$T> {
                match (self.0.entity(handle.entity())) {
                    Ok(entity) => entity.get::<$T>().map(|data| &*data),
                    Err(_) => None,
                }
            }
        }

        impl<'a> ComponentSet<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn size_hint(&self) -> usize {
                0
            }

            #[inline(always)]
            fn for_each(&self, mut f: impl FnMut(Index, &$T)) {
                self.0.query::<&$T>().iter().for_each(|(entity, data)| f(entity.handle(), data));
            }
        }

        impl<'a> ComponentSetMut<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn set_internal(&mut self, handle: Index, val: $T) {
                let _ = self.0.entity(handle.entity()).map(|entity| entity.get_mut::<&$T>().map(|data| **data = val));
            }

            #[inline(always)]
            fn map_mut_internal<Result>(
                &mut self,
                handle: Index,
                f: impl FnOnce(&mut $T) -> Result,
            ) -> Option<Result> {
                match (self.0.entity(handle.entity())) {
                    Ok(entity) => entity.get_mut::<$T>().map(|mut data| f(&mut data)),
                    Err(_) => None,
                }
            }
        }
    }
);

macro_rules! impl_component_set(
    ($ComponentsSet: ident, $T: ty) => {
        impl<'a> ComponentSetOption<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn get(&self, handle: Index) -> Option<&$T> {
                match (self.0.entity(handle.entity())) {
                    Ok(entity) => entity.get::<$T>().map(|data| &*data),
                    Err(_) => None,
                }
            }
        }

        impl<'a> ComponentSet<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn size_hint(&self) -> usize {
                0
            }

            #[inline(always)]
            fn for_each(&self, mut f: impl FnMut(Index, &$T)) {
                self.0.query::<&$T>().iter().for_each(|(entity, data)| f(entity.handle(), data));
            }
        }
    }
);

macro_rules! impl_component_set_option(
    ($ComponentsSet: ident, $T: ty) => {
        impl<'a> ComponentSetOption<$T> for $ComponentsSet<'a> {
            #[inline(always)]
            fn get(&self, handle: Index) -> Option<&$T> {
                match (self.0.entity(handle.entity())) {
                    Ok(entity) => entity.get::<$T>().map(|data| &*data),
                    Err(_) => None,
                }
            }
        }
    }
);

pub struct RigidBodyComponentsSet<'a>(&'a World);

impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyPosition);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyVelocity);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyMassProps);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyIds);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyForces);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyActivation);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyChanges);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyCcd);
impl_component_set_mut!(RigidBodyComponentsSet, RigidBodyColliders);

impl_component_set!(RigidBodyComponentsSet, RigidBodyDamping);
impl_component_set!(RigidBodyComponentsSet, RigidBodyDominance);
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

impl_component_set_mut!(ColliderComponentsSet, ColliderChanges);
impl_component_set_mut!(ColliderComponentsSet, ColliderPosition);
impl_component_set_mut!(ColliderComponentsSet, ColliderBroadPhaseData);

impl_component_set!(ColliderComponentsSet, ColliderShape);
impl_component_set!(ColliderComponentsSet, ColliderType);
impl_component_set!(ColliderComponentsSet, ColliderMaterial);
impl_component_set!(ColliderComponentsSet, ColliderFlags);

impl_component_set_option!(ColliderComponentsSet, ColliderParent);

#[derive(Query, Debug, PartialEq)]
struct RigidBodyComponentsQuery<'a>(
    &'a RigidBodyPosition,
    &'a RigidBodyVelocity,
    &'a RigidBodyMassProps,
    &'a RigidBodyIds,
    &'a RigidBodyForces,
    &'a RigidBodyActivation,
    &'a RigidBodyChanges,
    &'a RigidBodyCcd,
    &'a RigidBodyColliders,
    &'a RigidBodyDamping,
    &'a RigidBodyDominance,
    &'a RigidBodyType,
);

#[derive(Query)]
struct ColliderComponentsQuery<'a>(
    &'a ColliderChanges,
    &'a ColliderPosition,
    &'a ColliderBroadPhaseData,
    &'a ColliderShape,
    &'a ColliderType,
    &'a ColliderMaterial,
    &'a ColliderFlags,
    Option<&'a ColliderParent>,
);

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
