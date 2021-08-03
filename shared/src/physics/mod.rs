use hecs::{Bundle, Entity, Query, World};
use rapier2d::{
    data::{ComponentSet, ComponentSetMut, ComponentSetOption, Index},
    prelude::*,
};
use std::ops::Deref;

pub mod resources;
pub mod systems;

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
