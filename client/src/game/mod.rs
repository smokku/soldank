use crate::{
    calc::*,
    constants::*,
    cvars::Config,
    debug,
    engine::{input::InputEvent, world::WorldCameraExt, Engine, Game},
    game,
    mapfile::MapFile,
    mq,
    physics::*,
    render::{self as render, components::Camera, GameGraphics},
    soldier::Soldier,
    Weapon, WeaponKind,
};
use ::resources::Resources;
use gvfs::filesystem::Filesystem;
use hecs::{With, World};

pub mod components;
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
            render::components::Cursor::default(),
            render::components::Sprite::new("Crosshair", "38"),
        ));

        // run startup scripts
        let mut configs = Vec::new();
        let mut autoexecs = Vec::new();
        for file in self.filesystem.read_dir("/").unwrap() {
            if let Some(ext) = file.extension() {
                if ext == "cfg" {
                    if let Some(stem) = file.file_stem() {
                        let stem = stem.to_string_lossy();
                        if stem.starts_with("config") {
                            configs.push(file.to_string_lossy().to_string());
                        }
                        if stem.starts_with("autoexec") {
                            autoexecs.push(file.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        let mut configs = configs.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        let mut autoexecs = autoexecs.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        human_sort::sort(&mut configs);
        human_sort::sort(&mut autoexecs);
        for config in configs.iter().chain(autoexecs.iter()) {
            if self.filesystem.is_file(config) {
                log::info!("Loading {}", config);
                match eng.script.evaluate_file(
                    config,
                    eng.input,
                    &mut self.config,
                    &mut self.filesystem,
                    &mut self.world,
                ) {
                    Ok(_ctx) => log::debug!("Loaded {}", config),
                    Err(error) => log::error!("{}", error),
                }
            }
        }

        // spawn Player
        let map = self.resources.get::<MapFile>().unwrap();
        let soldier = Soldier::new(&map.spawnpoints[0], self.config.phys.gravity);
        let position = soldier.particle.pos;
        let player = self.world.spawn((
            // soldier,
            components::Pawn,
            components::Input::default(),
            render::components::Camera {
                zoom: self.config.debug.initial_zoom,
                ..Default::default()
            },
            render::components::Position(position),
            game::systems::ForceMovement,
            game::physics::PreviousPhysics::default(),
        ));
        self.world.make_active_camera(player).unwrap();
        self.world
            .insert(
                player,
                RigidBodyBundle {
                    body_type: RigidBodyType::Dynamic,
                    mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
                    position: (position / self.config.phys.scale).into(),
                    activation: RigidBodyActivation::cannot_sleep(),
                    forces: RigidBodyForces {
                        gravity_scale: 0.8,
                        ..Default::default()
                    },
                    ccd: RigidBodyCcd {
                        ccd_enabled: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        self.world
            .insert(
                player,
                ColliderBundle {
                    shape: ColliderShape::capsule(
                        Vec2::new(0., -7. / self.config.phys.scale).into(),
                        Vec2::new(0., 5. / self.config.phys.scale).into(),
                        3. / self.config.phys.scale,
                    ),
                    mass_properties: ColliderMassProps::Density(0.5),
                    material: ColliderMaterial::new(3.0, 0.1),
                    ..Default::default()
                },
            )
            .unwrap();
        let legs = self.world.spawn((
            components::Legs,
            Parent(player),
            game::physics::PreviousPhysics::default(),
        ));
        self.world
            .insert(
                legs,
                RigidBodyBundle {
                    position: (position / self.config.phys.scale).into(),
                    activation: RigidBodyActivation::cannot_sleep(),
                    damping: RigidBodyDamping {
                        angular_damping: 20.,
                        ..RigidBodyDamping::default()
                    },
                    ccd: RigidBodyCcd {
                        ccd_enabled: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        self.world
            .insert(
                legs,
                ColliderBundle {
                    shape: ColliderShape::ball(4.5 / self.config.phys.scale),
                    flags: ColliderFlags::from(ActiveHooks::FILTER_CONTACT_PAIRS),
                    mass_properties: ColliderMassProps::Density(0.3),
                    material: ColliderMaterial::new(0.0, 0.0),
                    ..Default::default()
                },
            )
            .unwrap();
        let mut legs_body_joint = BallJoint::new(
            Vec2::new(0.0, 0.0).into(),
            Vec2::new(0.0, 8.0 / self.config.phys.scale).into(),
        );
        legs_body_joint.motor_model = SpringModel::Disabled;
        self.world
            .spawn((JointBuilderComponent::new(legs_body_joint, legs, player),));
    }

    fn update(&mut self, eng: Engine<'_>) {
        if cfg!(debug_assertions) {
            debug::build_ui(&eng, self);
        }

        let screen_size = eng.quad_ctx.screen_size();
        let mouse_x = eng.input.mouse_x * GAME_WIDTH / screen_size.0;
        let mouse_y = eng.input.mouse_y * GAME_HEIGHT / screen_size.1;

        render::systems::update_cursor(&mut self.world, mouse_x, mouse_y);
        self::systems::apply_input(&mut self.world, &eng);

        for event in eng.input.drain_events() {
            #[allow(clippy::single_match)]
            match event {
                InputEvent::Key {
                    down,
                    keycode,
                    keymods,
                    repeat,
                } => match keycode {
                    mq::KeyCode::GraveAccent if down && !repeat && keymods.ctrl => {
                        self.config.debug.visible = !self.config.debug.visible;
                    }
                    mq::KeyCode::Tab if down && !repeat => {
                        // FIXME: remove this
                        let weapons = self.resources.get::<Vec<Weapon>>().unwrap();
                        for (_entity, mut soldier) in self
                            .world
                            .query::<With<game::components::Pawn, &mut Soldier>>()
                            .iter()
                        {
                            let index = soldier.primary_weapon().kind.index();
                            let index = (index + 1) % (WeaponKind::NoWeapon.index() + 1);
                            let active_weapon = soldier.active_weapon;
                            soldier.weapons[active_weapon] = weapons[index];
                        }
                    }
                    mq::KeyCode::Equal if down => {
                        for (_ent, mut camera) in self.world.query::<&mut Camera>().iter() {
                            if camera.is_active {
                                camera.zoom -= TIMESTEP_RATE as f32;
                            }
                        }
                    }
                    mq::KeyCode::Minus if down => {
                        for (_ent, mut camera) in self.world.query::<&mut Camera>().iter() {
                            if camera.is_active {
                                camera.zoom += TIMESTEP_RATE as f32;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {
                    // just drop it for now
                }
            }
        }

        eng.script.consume_events(
            eng.input,
            &mut self.config,
            &mut self.filesystem,
            &mut self.world,
        );

        game::systems::update_soldiers(
            &mut self.world,
            &self.resources,
            &self.config,
            (mouse_x, mouse_y),
        );

        game::systems::primitive_movement(&mut self.world);
        game::systems::force_movement(&mut self.world, &self.config);

        self.step_physics(eng.delta);

        self.config_update();

        game::physics::update_previous_physics(&mut self.world);

        self.world.clear_trackers();
        // self.resources.get_mut::<AppEventsQueue>().unwrap().clear();
    }

    fn draw(&mut self, eng: Engine<'_>) {
        render::debug::debug_render(
            eng.quad_ctx,
            &mut self.graphics,
            &self.world,
            &self.resources,
            &self.config,
        );

        self.graphics.render_frame(
            &mut self.context,
            eng.quad_ctx,
            &self.world,
            &self.resources,
            &self.config,
            // self.last_frame - TIMESTEP_RATE * (1.0 - p),
            eng.overstep_percentage,
        );
    }
}
