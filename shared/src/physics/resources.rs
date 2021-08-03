use super::*;
use hecs::Changed;
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
        world.clear_trackers::<RigidBodyActivation>();
        world.clear_trackers::<RigidBodyPosition>();
        world.clear_trackers::<RigidBodyVelocity>();
        world.clear_trackers::<RigidBodyForces>();
        world.clear_trackers::<RigidBodyType>();
        world.clear_trackers::<RigidBodyColliders>();

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
        world.clear_trackers::<ColliderPosition>();
        world.clear_trackers::<ColliderFlags>();
        world.clear_trackers::<ColliderShape>();
        world.clear_trackers::<ColliderType>();
        world.clear_trackers::<ColliderParent>();
    }
}
