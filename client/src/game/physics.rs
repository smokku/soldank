use crate::physics::*;
use hecs::World;

pub struct SameParentFilter;
impl PhysicsHooksWithWorld for SameParentFilter {
    fn filter_contact_pair(
        &self,
        context: &PairFilterContext<RigidBodyComponentsSet, ColliderComponentsSet>,
        world: &World,
    ) -> Option<SolverFlags> {
        // if collider1 and collider2 have the same Parent() component
        // or collider1 is Parent() of collider2
        // or collider2 is Parent() of collider1
        // then there is no contact

        let collider1 = context.collider1.entity();
        let collider2 = context.collider2.entity();
        let parent1 = world.get::<Parent>(collider1).ok();
        let parent2 = world.get::<Parent>(collider2).ok();

        if let Some(parent1) = parent1 {
            if let Some(parent2) = parent2 {
                if **parent1 == **parent2 || **parent1 == collider2 || **parent2 == collider1 {
                    return None;
                }
            } else if **parent1 == collider2 {
                return None;
            }
        } else if let Some(parent2) = parent2 {
            if **parent2 == collider1 {
                return None;
            }
        }

        Some(SolverFlags::all())
    }
}

#[derive(Debug)]
pub struct PreviousPhysics {
    pub position: Isometry<Real>,
    pub linvel: Vector<Real>,
    pub angvel: AngVector<Real>,
    pub force: Vector<Real>,
    pub torque: AngVector<Real>,
}

impl Default for PreviousPhysics {
    fn default() -> Self {
        Self {
            position: Isometry::identity(),
            linvel: Default::default(),
            angvel: Default::default(),
            force: Default::default(),
            torque: Default::default(),
        }
    }
}

pub fn update_previous_physics(world: &mut World) {
    for (_entity, (mut prev, pos, vel, force)) in world
        .query::<(
            &mut PreviousPhysics,
            Option<&RigidBodyPosition>,
            Option<&RigidBodyVelocity>,
            Option<&RigidBodyForces>,
        )>()
        .iter()
    {
        if let Some(pos) = pos {
            prev.position = pos.position;
        }
        if let Some(vel) = vel {
            prev.linvel = vel.linvel;
            prev.angvel = vel.angvel;
        }
        if let Some(force) = force {
            prev.force = force.force;
            prev.torque = force.torque;
        }
    }
}
