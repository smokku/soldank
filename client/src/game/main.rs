use super::GameState;
use crate::{
    calc::*,
    constants::*,
    debug,
    engine::{input::InputEvent, world::WorldCameraExt, Engine, Game},
    game::{self as game, components, physics},
    mapfile::MapFile,
    mq,
    physics::*,
    render::{self as render, components::Camera},
    soldier::Soldier,
    Weapon, WeaponKind,
};
use enumflags2::BitFlags;
use hecs::With;

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
            game::physics::PreviousPhysics::default(),
            Soldier::new(
                &crate::MapSpawnpoint {
                    active: false,
                    x: position.x as i32,
                    y: position.y as i32,
                    team: 0,
                },
                0.,
            ),
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
                        Vec2::new(0., -9. / self.config.phys.scale).into(),
                        Vec2::new(0., 7. / self.config.phys.scale).into(),
                        3. / self.config.phys.scale,
                    ),
                    mass_properties: ColliderMassProps::Density(0.5),
                    material: ColliderMaterial::new(3.0, 0.1),
                    flags: ColliderFlags {
                        collision_groups: physics::InteractionGroups::new(
                            BitFlags::<physics::InteractionFlag>::from(
                                physics::InteractionFlag::Player,
                            )
                            .bits(),
                            BitFlags::<physics::InteractionFlag>::all().bits(),
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        let legs = self.world.spawn((
            components::Legs,
            Parent(player),
            game::physics::PreviousPhysics::default(),
            game::physics::Contact::default(),
        ));
        self.world
            .insert(
                legs,
                RigidBodyBundle {
                    position: (position / self.config.phys.scale).into(),
                    activation: RigidBodyActivation::cannot_sleep(),
                    mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
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
                    flags: ColliderFlags {
                        collision_groups: physics::InteractionGroups::new(
                            BitFlags::<physics::InteractionFlag>::from(
                                physics::InteractionFlag::Player,
                            )
                            .bits(),
                            BitFlags::<physics::InteractionFlag>::all().bits(),
                        ),
                        active_events: ActiveEvents::CONTACT_EVENTS,
                        active_hooks: ActiveHooks::FILTER_CONTACT_PAIRS,
                        ..Default::default()
                    },
                    material: ColliderMaterial::new(10.0, 0.0),
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
        game::systems::apply_input(&mut self.world, &eng);

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

        game::systems::primitive_movement(&mut self.world);
        game::systems::force_movement(&mut self.world, &self.config);
        game::systems::soldier_movement(
            &mut self.world,
            &self.resources,
            &self.config,
            (mouse_x, mouse_y),
        );

        self.step_physics(eng.delta);

        self.config_update();

        game::physics::update_previous_physics(&mut self.world);
        game::physics::process_contact_events(&mut self.world, &self.resources);
        game::systems::follow_camera(&mut self.world, &self.config);
        game::systems::update_soldiers(&mut self.world, &self.resources, &self.config);

        self.world.clear_trackers();
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
