use crate::{
    cvars::Config,
    engine::{Engine, Game},
    physics::*,
    render::{self as render, GameGraphics},
};
use ::resources::Resources;
use gvfs::filesystem::Filesystem;
use hecs::World;

pub mod components;
mod main;
pub mod physics;
pub mod systems;

pub struct GameState {
    pub world: World,
    pub resources: Resources,
    pub filesystem: Filesystem,
    pub config: Config,

    context: gfx2d::Gfx2dContext,
    graphics: GameGraphics,
}

impl GameState {
    pub(crate) fn new(
        context: gfx2d::Gfx2dContext,
        world: World,
        resources: Resources,
        filesystem: Filesystem,
        config: Config,
    ) -> Self {
        GameState {
            context,
            graphics: GameGraphics::new(),
            world,
            resources,
            filesystem,
            config,
        }
    }

    pub fn config_update(&self) {
        // let app_events = self.resources.get::<AppEventsQueue>().unwrap();
        // if app_events
        //     .iter()
        //     .any(|event| matches!(event, AppEvent::CvarsChanged))
        // {
        //     let dt = self
        //         .resources
        //         .get::<Config>()
        //         .unwrap()
        //         .net
        //         .orb
        //         .read()
        //         .unwrap()
        //         .timestep_seconds as f32;
        // }
    }

    fn step_physics(&mut self, delta: f64) {
        use crate::physics::*;
        let gravity = vector![0.0, 9.81];

        // let configuration = resources.get::<RapierConfiguration>().unwrap();
        let mut integration_parameters = self.resources.get_mut::<IntegrationParameters>().unwrap();
        integration_parameters.dt = delta as f32;
        let mut modifs_tracker = self.resources.get_mut::<ModificationTracker>().unwrap();

        let mut physics_pipeline = self.resources.get_mut::<PhysicsPipeline>().unwrap();
        // let mut query_pipeline = self.resources.get_mut::<QueryPipeline>().unwrap();
        let mut island_manager = self.resources.get_mut::<IslandManager>().unwrap();
        let mut broad_phase = self.resources.get_mut::<BroadPhase>().unwrap();
        let mut narrow_phase = self.resources.get_mut::<NarrowPhase>().unwrap();
        let mut ccd_solver = self.resources.get_mut::<CCDSolver>().unwrap();
        let mut joint_set = self.resources.get_mut::<JointSet>().unwrap();
        let mut joints_entity_map = self.resources.get_mut::<JointsEntityMap>().unwrap();
        const PHYSICS_HOOKS: physics::SameParentFilter = physics::SameParentFilter {};
        let event_handler = ();

        // TODO: make all preparations before looping necessary number of steps
        attach_bodies_and_colliders(&mut self.world);
        create_joints(&mut self.world, &mut joint_set, &mut joints_entity_map);
        finalize_collider_attach_to_bodies(&mut self.world, &mut modifs_tracker);

        prepare_step(&mut self.world, &mut modifs_tracker);

        step_world(
            &mut self.world,
            &gravity,
            &integration_parameters,
            &mut physics_pipeline,
            &mut modifs_tracker,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut joint_set,
            &mut joints_entity_map,
            &mut ccd_solver,
            &PHYSICS_HOOKS,
            &event_handler,
        );

        despawn_outliers(&mut self.world, 2500., self.config.phys.scale);
        collect_removals(&mut self.world, &mut modifs_tracker);
    }
}
