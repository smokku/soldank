use super::*;
use hecs::{Changed, Entity};
use std::collections::HashMap;

pub struct ModificationTracker {
    pub(crate) modified_bodies: Vec<RigidBodyHandle>,
    pub(crate) modified_colliders: Vec<ColliderHandle>,
    pub(crate) removed_bodies: Vec<RigidBodyHandle>,
    pub(crate) removed_colliders: Vec<ColliderHandle>,
    // NOTE: right now, this actually contains an Entity instead of the JointHandle.
    //       but we will switch to JointHandle soon.
    pub(crate) removed_joints: Vec<JointHandle>,
    // We need to maintain these two because we have to access them
    // when an entity containing a collider/rigid-body has been despawn.
    pub(crate) body_colliders: HashMap<RigidBodyHandle, Vec<ColliderHandle>>,
    pub(crate) colliders_parent: HashMap<ColliderHandle, RigidBodyHandle>,
}

impl Default for ModificationTracker {
    fn default() -> Self {
        Self {
            modified_bodies: vec![],
            modified_colliders: vec![],
            removed_bodies: vec![],
            removed_colliders: vec![],
            removed_joints: vec![],
            body_colliders: HashMap::new(),
            colliders_parent: HashMap::new(),
        }
    }
}

impl ModificationTracker {
    pub fn clear_modified_and_removed(&mut self) {
        self.modified_colliders.clear();
        self.modified_bodies.clear();
        self.removed_bodies.clear();
        self.removed_colliders.clear();
        self.removed_joints.clear();
    }

    pub fn detect_modifications(&mut self, world: &mut World) {
        // Detect modifications.
        for (
            entity,
            (
                mut rb_changes,
                mut rb_activation,
                rb_activation_ch,
                rb_pos_ch,
                rb_vels_ch,
                rb_forces_ch,
                rb_type_ch,
                rb_colliders_ch,
            ),
        ) in world
            .query::<(
                &mut RigidBodyChanges,
                &mut RigidBodyActivation,
                Changed<RigidBodyActivation>,
                Changed<RigidBodyPosition>,
                Changed<RigidBodyVelocity>,
                Changed<RigidBodyForces>,
                Changed<RigidBodyType>,
                Changed<RigidBodyColliders>,
            )>()
            .iter()
        {
            if rb_activation_ch
                || rb_pos_ch
                || rb_vels_ch
                || rb_forces_ch
                || rb_type_ch
                || rb_colliders_ch
            {
                if !rb_changes.contains(RigidBodyChanges::MODIFIED) {
                    self.modified_bodies.push(entity.handle());
                }

                *rb_changes |= RigidBodyChanges::MODIFIED;

                if rb_pos_ch {
                    *rb_changes |= RigidBodyChanges::POSITION;
                }
                if rb_type_ch {
                    *rb_changes |= RigidBodyChanges::TYPE;
                }
                if rb_colliders_ch {
                    *rb_changes |= RigidBodyChanges::COLLIDERS;
                }

                // Wake-up the rigid-body.
                *rb_changes |= RigidBodyChanges::SLEEP;
                rb_activation.wake_up(true);
            }
        }

        for (
            entity,
            (mut co_changes, co_pos_ch, co_groups_ch, co_shape_ch, co_type_ch, co_parent_ch),
        ) in world
            .query::<(
                &mut ColliderChanges,
                Changed<ColliderPosition>,
                Changed<ColliderFlags>,
                Changed<ColliderShape>,
                Changed<ColliderType>,
                Option<Changed<ColliderParent>>,
            )>()
            .iter()
        {
            if co_pos_ch
                || co_groups_ch
                || co_shape_ch
                || co_type_ch
                || co_parent_ch.unwrap_or(false)
            {
                if !co_changes.contains(ColliderChanges::MODIFIED) {
                    self.modified_colliders.push(entity.handle());
                }

                *co_changes |= ColliderChanges::MODIFIED;

                if co_pos_ch {
                    *co_changes |= ColliderChanges::POSITION;
                }
                if co_groups_ch {
                    *co_changes |= ColliderChanges::GROUPS;
                }
                if co_shape_ch {
                    *co_changes |= ColliderChanges::SHAPE;
                }
                if co_type_ch {
                    *co_changes |= ColliderChanges::TYPE;
                }
                if co_parent_ch == Some(true) {
                    *co_changes |= ColliderChanges::PARENT;
                }
            }
        }
    }

    pub fn detect_removals(&mut self, world: &mut World) {
        self.removed_bodies.extend(
            world
                .removed::<RigidBodyChanges>()
                .iter()
                .map(|e| IntoHandle::<RigidBodyHandle>::handle(*e)),
        );
        self.removed_colliders.extend(
            world
                .removed::<ColliderChanges>()
                .iter()
                .map(|e| IntoHandle::<ColliderHandle>::handle(*e)),
        );
        // self.removed_joints.extend(
        //     world
        //         .removed::<JointHandleComponent>()
        //         .iter()
        //         .map(|e| IntoHandle::<JointHandle>::handle(*e)),
        // );
    }

    pub fn propagate_removals<Bodies>(
        &mut self,
        _islands: &mut IslandManager,
        bodies: &mut Bodies,
        // joints: &mut JointSet,
        // joints_map: &mut JointsEntityMap,
    ) -> Vec<Entity>
    where
        Bodies: ComponentSetMut<RigidBodyChanges>
            + ComponentSetMut<RigidBodyColliders>
            + ComponentSetMut<RigidBodyActivation> // Needed for joint removal.
            + ComponentSetMut<RigidBodyIds> // Needed for joint removal.
            + ComponentSet<RigidBodyType>, // Needed for joint removal.
    {
        let mut cleanup_entities = Vec::new();
        for removed_body in self.removed_bodies.iter() {
            if let Some(colliders) = self.body_colliders.remove(removed_body) {
                for collider in colliders {
                    cleanup_entities.push(collider.entity());
                    self.removed_colliders.push(collider);
                }
            }

            // let mut removed_joints =
            //     joints.remove_joints_attached_to_rigid_body(*removed_body, islands, bodies);
            // self.removed_joints.append(&mut removed_joints);
        }

        for removed_collider in self.removed_colliders.iter() {
            if let Some(parent) = self.colliders_parent.remove(removed_collider) {
                let rb_changes: Option<RigidBodyChanges> = bodies.get(parent.0).copied();

                if let Some(mut rb_changes) = rb_changes {
                    // Keep track of the fact the rigid-body will be modified.
                    if !rb_changes.contains(RigidBodyChanges::MODIFIED) {
                        self.modified_bodies.push(parent);
                    }

                    // Detach the collider from the rigid-body.
                    bodies.map_mut_internal(parent.0, |rb_colliders: &mut RigidBodyColliders| {
                        rb_colliders.detach_collider(&mut rb_changes, *removed_collider);
                    });

                    // Set the new rigid-body changes flags.
                    bodies.set_internal(parent.0, rb_changes);

                    // Update the body's colliders map `self.body_colliders`.
                    let body_colliders = self.body_colliders.get_mut(&parent).unwrap();
                    if let Some(i) = body_colliders.iter().position(|c| *c == *removed_collider) {
                        body_colliders.swap_remove(i);
                    }
                }
            }
        }

        // for removed_joints in self.removed_joints.iter() {
        //     let joint_handle = joints_map.0.remove(&removed_joints.entity());
        //     if let Some(joint_handle) = joint_handle {
        //         joints.remove(joint_handle, islands, bodies, true);
        //     }
        // }

        cleanup_entities
    }
}
