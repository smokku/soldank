#[macro_use]
extern crate clap;

macro_rules! iif(
    ($cond:expr, $then:expr, $else:expr) => (if $cond { $then } else { $else })
);

mod anims;
mod bullet;
mod calc;
mod constants;
mod control;
mod cvars;
mod debug;
mod engine;
mod events;
mod game;
mod mapfile;
mod networking;
mod particles;
mod physics;
mod render;
mod soldier;
mod state;
mod systems;
mod weapons;

use anims::*;
use bullet::*;
use calc::*;
use constants::*;
use control::*;
use events::*;
use mapfile::*;
use networking::*;
use particles::*;
use render::*;
use soldier::*;
use state::*;
use weapons::*;

use cvars::{set_cli_cvars, Config, NetConfig};
use gfx2d::{math, mq};
use gvfs::filesystem::{File, Filesystem};
use hecs::World;
use quad_rand as rand;
use resources::Resources;
use std::{
    env, path,
    sync::{Arc, RwLock},
};

use soldank_shared::{networking::GameWorld, orb};

fn main() {
    color_eyre::install().unwrap();
    env_logger::init();

    let cmd = clap::app_from_crate!()
        .arg(
            clap::Arg::with_name("map")
                .help("name of map to load")
                .short("m")
                .long("map")
                .takes_value(true)
                .default_value(DEFAULT_MAP),
        )
        .arg(
            clap::Arg::with_name("debug")
                .help("display debug UI on start (^` to toggle)")
                .long("debug"),
        )
        .arg(
            clap::Arg::with_name("connect")
                .value_name("address:port")
                .help("server address and port to connect")
                .short("c")
                .long("connect")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("key")
                .help("server connection key")
                .short("k")
                .long("key")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("nick")
                .help("user nickname")
                .short("n")
                .long("nick")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("set")
                .help("set cvar value [multiple]")
                .long("set")
                .takes_value(true)
                .multiple(true)
                .number_of_values(2)
                .value_names(&["cvar", "value"]),
        )
        .get_matches();

    let mut filesystem = Filesystem::new(clap::crate_name!(), "Soldat2k").unwrap();

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        filesystem.mount(path.as_path(), true);
    }
    log::info!("Full VFS info: {:#?}", filesystem);

    let mut mods = Vec::new();

    let soldat_smod = path::Path::new("/soldat.smod");
    if filesystem.is_file(soldat_smod) {
        mods.push((
            filesystem.open(soldat_smod).unwrap(),
            soldat_smod.to_string_lossy().to_string(),
        ));
    }

    for f in filesystem.read_dir(path::Path::new("/")).unwrap() {
        let f = f.as_path();
        if let Some(name) = f.to_str() {
            if filesystem.is_file(f) && f != soldat_smod && name.ends_with(".smod") {
                mods.push((filesystem.open(f).unwrap(), name.to_string()));
            }
        }
    }
    for (md, path) in mods.drain(..) {
        match md {
            File::VfsFile(file) => {
                filesystem.add_zip_file(file).unwrap_or_else(|err| {
                    panic!(
                        "Failed to add `{}` file to VFS. (Make sure it is a proper ZIP file.): {}",
                        path, err
                    )
                });
            }
        }
    }
    // filesystem.print_all();

    let mut networking = Networking::new(cmd.value_of("connect"));
    if let Some(key) = cmd.value_of("key") {
        networking.connection_key = key.to_string();
    }
    if let Some(nick) = cmd.value_of("nick") {
        networking.nick_name = nick.to_string();
    }

    let mut map_name = cmd.value_of("map").unwrap_or(DEFAULT_MAP).to_owned();
    map_name.push_str(".pms");

    let map = MapFile::load_map_file(&mut filesystem, map_name.as_str());
    log::info!("Using map: {}", map.mapname);

    let mut config = Config {
        net: NetConfig {
            orb: Arc::new(RwLock::new(orb::Config {
                timestep_seconds: TIMESTEP_RATE,
                ..Default::default()
            })),
            ..Default::default()
        },
        ..Default::default()
    };
    config.debug.visible = cmd.is_present("debug");
    set_cli_cvars(&mut config, &cmd);

    let state = MainState {
        game_width: WINDOW_WIDTH as f32 * (480.0 / WINDOW_HEIGHT as f32),
        game_height: 480.0,
        camera: Vec2::ZERO,
        camera_prev: Vec2::ZERO,
        mouse: Vec2::ZERO,
        mouse_prev: Vec2::ZERO,
        mouse_phys: Vec2::ZERO,
        mouse_pressed: false,
        zoom: 0.0,
        mouse_over_ui: false,
    };

    AnimData::initialize(&mut filesystem);
    Soldier::initialize(&mut filesystem, &config);

    let weapons: Vec<Weapon> = WeaponKind::values()
        .iter()
        .map(|k| Weapon::new(*k, false))
        .collect();

    let client = orb::client::Client::<GameWorld>::new(mq::date::now(), config.net.orb.clone());

    let mut world = World::new();

    let mut resources = Resources::new();

    let app_events = AppEventsQueue::new();

    resources.insert(app_events);
    resources.insert(map);
    // resources.insert(config);
    resources.insert(state);
    resources.insert(weapons);

    resources.insert(physics::PhysicsPipeline::new());
    // resources.insert(physics::QueryPipeline::new());
    // resources.insert(physics::RapierConfiguration::default());
    resources.insert(physics::IntegrationParameters::default());
    resources.insert(physics::BroadPhase::new());
    resources.insert(physics::NarrowPhase::new());
    resources.insert(physics::IslandManager::new());
    resources.insert(physics::JointSet::new());
    resources.insert(physics::CCDSolver::new());
    // resources.insert(physics::Events::<IntersectionEvent>::default());
    // resources.insert(physics::Events::<ContactEvent>::default());
    // resources.insert(physics::SimulationToRenderTime::default());
    // resources.insert(physics::JointsEntityMap::default());
    resources.insert(physics::ModificationTracker::default());
    physics::create_map_colliders(&mut world, &resources, &config);

    let conf = mq::conf::Conf {
        sample_count: 4,
        window_title: clap::crate_name!().to_string(),
        window_width: WINDOW_WIDTH as _,
        window_height: WINDOW_HEIGHT as _,
        ..Default::default()
    };
    mq::start(conf, |mut ctx| {
        let context = gfx2d::Gfx2dContext::new(&mut ctx);
        mq::UserData::owning(
            engine::Runner::new(
                &mut ctx,
                game::GameState::new(context, world, resources, filesystem, config),
            ),
            ctx,
        )
    });
}

pub struct GameStage {
    world: World,
    resources: Resources,
    filesystem: Filesystem,
    networking: Networking,
    client: orb::client::Client<GameWorld>,

    // context: gfx2d::Gfx2dContext,
    // graphics: GameGraphics,
    last_frame: f64,
    timeacc: f64,

    soldier: Soldier,
    emitter: Vec<EmitterItem>,
    bullets: Vec<Bullet>,

    zoomin_pressed: bool,
    zoomout_pressed: bool,
}

impl GameStage {
    pub fn new(
        ctx: &mut mq::Context,
        world: World,
        mut resources: Resources,
        mut filesystem: Filesystem,
        networking: Networking,
        config: &Config,
        client: orb::client::Client<GameWorld>,
    ) -> Self {
        // setup window, renderer & main loop

        let mut state = resources.get_mut::<MainState>().unwrap();
        let map = resources.get::<MapFile>().unwrap();
        let soldier = Soldier::new(&map.spawnpoints[0], &config);
        state.camera = soldier.particle.pos;
        state.camera_prev = state.camera;

        drop(map);
        drop(state);
        // drop(config);

        GameStage {
            world,
            resources,
            filesystem,
            networking,
            client,

            last_frame: mq::date::now(),
            timeacc: 0.0,

            soldier,
            emitter: Vec::new(),
            bullets: Vec::new(),

            zoomin_pressed: false,
            zoomout_pressed: false,
        }
    }
}

impl mq::EventHandler for GameStage {
    fn update(&mut self, _ctx: &mut mq::Context) {
        physics::attach_bodies_and_colliders(&mut self.world);
        // physics::create_joints_system();
        physics::finalize_collider_attach_to_bodies(
            &mut self.world,
            &mut *self
                .resources
                .get_mut::<physics::ModificationTracker>()
                .unwrap(),
        );

        self.networking.tick += 1;
        self.networking.update();

        {
            let mut modifs_tracker = self
                .resources
                .get_mut::<physics::ModificationTracker>()
                .unwrap();
            physics::prepare_step(&mut self.world, &mut modifs_tracker);
        }

        let time = mq::date::now();
        self.timeacc += time - self.last_frame;
        self.last_frame = time;

        while self.timeacc >= TIMESTEP_RATE {
            self.timeacc -= TIMESTEP_RATE;

            {
                use physics::*;
                let gravity = vector![0.0, 9.81];

                // let configuration = resources.get::<RapierConfiguration>().unwrap();
                let integration_parameters = self.resources.get::<IntegrationParameters>().unwrap();
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

                physics::step_world(
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
            }

            // remove inactive bullets
            let mut i = 0;
            while i < self.bullets.len() {
                if !self.bullets[i].active {
                    self.bullets.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            // update soldiers
            self.soldier.update(
                &self.resources,
                &mut self.emitter,
                &*self.resources.get::<Config>().unwrap(),
            );

            // update bullets
            for bullet in self.bullets.iter_mut() {
                bullet.update(&self.resources);
            }

            // create emitted objects
            for item in self.emitter.drain(..) {
                match item {
                    EmitterItem::Bullet(params) => self.bullets.push(Bullet::new(
                        &params,
                        &*self.resources.get::<Config>().unwrap(),
                    )),
                };
            }

            {
                // update camera
                let mut state = self.resources.get_mut::<MainState>().unwrap();

                state.camera_prev = state.camera;
                state.mouse_prev = state.mouse;

                if self.zoomin_pressed ^ self.zoomout_pressed {
                    state.zoom += iif!(self.zoomin_pressed, -1.0, 1.0) * TIMESTEP_RATE as f32;
                }

                state.camera = {
                    let z = f32::exp(state.zoom);
                    let mut m = Vec2::ZERO;

                    m.x = z * (state.mouse.x - state.game_width / 2.0) / 7.0
                        * ((2.0 * 640.0 / state.game_width - 1.0)
                            + (state.game_width - 640.0) / state.game_width * 0.0 / 6.8);
                    m.y = z * (state.mouse.y - state.game_height / 2.0) / 7.0;

                    let mut cam_v = state.camera;
                    let p = self.soldier.particle.pos;
                    let norm = p - cam_v;
                    let s = norm * 0.14;
                    cam_v += s;
                    cam_v += m;
                    cam_v
                };
            }

            let time = mq::date::now();
            self.timeacc += time - self.last_frame;
            self.last_frame = time;
        }

        // systems::rotate_balls(&mut world, self.last_frame);

        self.networking.set_input_state(&self.soldier.control);

        self.networking.process(&self.resources, &mut self.client);
        self.client.update(self.timeacc, self.last_frame);
        if let Some(state) = self.client.display_state() {
            log::trace!("client_display_state: {}", state.display_state().len());
        }
        self.networking
            .post_process(&*self.resources.get::<Config>().unwrap());

        physics::despawn_outliers(
            &mut self.world,
            2500.,
            self.resources.get::<Config>().unwrap().phys.scale,
        );
        physics::collect_removals(
            &mut self.world,
            &mut *self
                .resources
                .get_mut::<physics::ModificationTracker>()
                .unwrap(),
        );
        // physics::config_update(&self.resources);

        self.world.clear_trackers();
        self.resources.get_mut::<AppEventsQueue>().unwrap().clear();
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            mq::KeyCode::Escape => ctx.request_quit(),
            mq::KeyCode::Equal => {
                self.zoomin_pressed = true;
            }
            mq::KeyCode::Minus => {
                self.zoomout_pressed = true;
            }
            mq::KeyCode::Tab => {
                let weapons = self.resources.get::<Vec<Weapon>>().unwrap();
                let index = self.soldier.primary_weapon().kind.index();
                let index = (index + 1) % (WeaponKind::NoWeapon.index() + 1);
                self.soldier.weapons[self.soldier.active_weapon] = weapons[index];
            }
            mq::KeyCode::GraveAccent => {
                if keymods.ctrl {
                    let mut config = self.resources.get_mut::<Config>().unwrap();
                    config.debug.visible = !config.debug.visible;
                }
            }

            _ => self.soldier.update_keys(true, keycode),
        }
    }

    fn key_up_event(
        &mut self,
        ctx: &mut gfx2d::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
    ) {
        match keycode {
            mq::KeyCode::Escape => ctx.request_quit(),
            mq::KeyCode::Equal => {
                self.zoomin_pressed = false;
            }
            mq::KeyCode::Minus => {
                self.zoomout_pressed = false;
            }
            _ => {
                let state = self.resources.get::<MainState>().unwrap();
                if !state.mouse_over_ui {
                    self.soldier.update_keys(false, keycode)
                }
            }
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        let mut state = self.resources.get_mut::<MainState>().unwrap();
        state.mouse_pressed = true;
        if !state.mouse_over_ui {
            self.soldier.update_mouse_button(true, button);
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut gfx2d::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.soldier.update_mouse_button(false, button);
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        let mut state = self.resources.get_mut::<MainState>().unwrap();
        state.mouse.x = x * state.game_width / WINDOW_WIDTH as f32;
        state.mouse.y = y * state.game_height / WINDOW_HEIGHT as f32;
        state.mouse_phys.x = x;
        state.mouse_phys.y = y;
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {}

    fn draw(&mut self, ctx: &mut mq::Context) {
        let p = f64::min(1.0, f64::max(0.0, self.timeacc / TIMESTEP_RATE));

        // self.egui_mq.begin_frame(ctx);
        // if cfg!(debug_assertions) {
        //     debug::build_ui(
        //         ctx,
        //         self.egui_mq.egui_ctx(),
        //         &mut self.world,
        //         &self.resources,
        //         self.last_frame as u64,
        //         p as f32,
        //     );
        // }
        // self.egui_mq.end_frame(ctx);

        // render::debug::debug_render(ctx, &mut self.graphics, &self.world, &self.resources);

        // self.graphics.render_frame(
        //     &mut self.context,
        //     ctx,
        //     &self.world,
        //     &self.resources,
        //     &self.soldier,
        //     &self.bullets,
        //     self.last_frame - TIMESTEP_RATE * (1.0 - p),
        //     p as f32,
        // );

        ctx.commit_frame();
    }
}
