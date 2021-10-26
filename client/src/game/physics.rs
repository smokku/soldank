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
