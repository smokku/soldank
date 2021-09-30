use crate::{
    components,
    constants::*,
    cvars::Config,
    debug,
    engine::{world::WorldCameraExt, Engine, Game},
    mapfile::MapFile,
    render::{self as render, systems, GameGraphics},
};
use gfx2d::math::Vec2;
use gvfs::filesystem::Filesystem;
use hecs::World;
use resources::Resources;

pub struct GameState {
    pub world: World,
    pub resources: Resources,
    pub filesystem: Filesystem,

    context: gfx2d::Gfx2dContext,
    graphics: GameGraphics,
}

impl GameState {
    pub(crate) fn new(
        context: gfx2d::Gfx2dContext,
        world: World,
        resources: Resources,
        filesystem: Filesystem,
    ) -> Self {
        GameState {
            context,
            graphics: GameGraphics::new(),
            world,
            resources,
            filesystem,
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
        // let mut joints_entity_map = self.resources.get_mut::<JointsEntityMap>().unwrap();
        // let physics_hooks = ();
        let event_handler = ();

        attach_bodies_and_colliders(&mut self.world);
        // create_joints_system();
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
            &mut ccd_solver,
            &event_handler,
        );

        despawn_outliers(
            &mut self.world,
            2500.,
            self.resources.get::<Config>().unwrap().phys.scale,
        );
        collect_removals(&mut self.world, &mut modifs_tracker);
    }
}

impl Game for GameState {
    fn initialize(&mut self, eng: Engine<'_>) {
        eng.quad_ctx.show_mouse(false);
        eng.quad_ctx.set_cursor_grab(true);

        let map = self.resources.get::<MapFile>().unwrap();
        self.graphics
            .load_sprites(eng.quad_ctx, &mut self.filesystem);
        self.graphics
            .load_map(eng.quad_ctx, &mut self.filesystem, &*map);

        // spawn cursor sprite
        self.world.spawn((
            components::Cursor::default(),
            components::Sprite::new("Crosshair", "38"),
        ));

        // spawn camera
        let camera = self.world.spawn((
            components::Camera::default(),
            components::Position::default(),
        ));
        self.world.make_active_camera(camera).unwrap();
    }

    fn update(&mut self, eng: Engine<'_>) {
        if cfg!(debug_assertions) {
            debug::build_ui(&eng, self);
        }

        let screen_size = eng.quad_ctx.screen_size();
        let mouse_x = eng.input.mouse_x * GAME_WIDTH / screen_size.0;
        let mouse_y = eng.input.mouse_y * GAME_HEIGHT / screen_size.1;

        systems::update_cursor(&mut self.world, mouse_x, mouse_y);

        for _event in eng.input.drain_events() {
            // just drop it for now
        }

        self.step_physics(eng.delta);

        self.config_update();

        self.world.clear_trackers();
        // self.resources.get_mut::<AppEventsQueue>().unwrap().clear();
    }

    fn draw(&mut self, eng: Engine<'_>) {
        render::debug::debug_render(
            eng.quad_ctx,
            &mut self.graphics,
            &self.world,
            &self.resources,
        );

        self.graphics.render_frame(
            &mut self.context,
            eng.quad_ctx,
            &self.world,
            &self.resources,
            // self.last_frame - TIMESTEP_RATE * (1.0 - p),
            eng.overstep_percentage,
        );
    }
}
