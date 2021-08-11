use crate::components::Position;
use hecs::World;
pub use hecs_rapier::*;

pub struct PhysicsEngine {
    pub(crate) gravity: Vector<Real>,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) physics_pipeline: PhysicsPipeline,
    pub(crate) modification_tracker: ModificationTracker,
    pub(crate) island_manager: IslandManager,
    pub(crate) broad_phase: BroadPhase,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) joint_set: JointSet,
    pub(crate) ccd_solver: CCDSolver,
    // pub(crate) physics_hooks: dyn PhysicsHooks<RigidBodyComponentsSet, ColliderComponentsSet>,
    // pub(crate) event_handler: dyn EventHandler,
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        PhysicsEngine {
            gravity: vector![0.0, 9.81],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            modification_tracker: ModificationTracker::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }
}

pub fn despawn_outliers(world: &mut World, max_pos: f32, phys_scale: f32) {
    let mut to_despawn = Vec::new();

    for (entity, pos) in world.query::<&RigidBodyPosition>().iter() {
        let x = pos.position.translation.x * phys_scale;
        let y = pos.position.translation.y * phys_scale;
        if !(-max_pos..=max_pos).contains(&x) || !(-max_pos..=max_pos).contains(&y) {
            to_despawn.push(entity);
        }
    }

    for (entity, pos) in world.query::<&Position>().iter() {
        if pos.x > max_pos || pos.x < -max_pos || pos.y > max_pos || pos.y < -max_pos {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        world.despawn(entity).unwrap();
    }
}
