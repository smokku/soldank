#[macro_use]
extern crate clap;

macro_rules! iif(
    ($cond:expr, $then:expr, $else:expr) => (if $cond { $then } else { $else })
);

mod anims;
mod bullet;
mod calc;
mod components;
mod constants;
mod control;
mod cvars;
mod debug;
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
use mapfile::*;
use networking::*;
use particles::*;
use render::*;
use soldier::*;
use state::*;
use weapons::*;

use cvars::{set_cli_cvars, Config, NetConfig};
use gfx2d::macroquad::{self as macroquad, prelude as mq};
use gvfs::filesystem::{File, Filesystem};
use hecs::World;
use resources::Resources;
use std::{
    env, path,
    sync::{Arc, RwLock},
};

use soldank_shared::{networking::MyWorld, orb};

fn config() -> mq::Conf {
    mq::Conf {
        sample_count: 4,
        window_title: clap::crate_name!().to_string(),
        window_width: WINDOW_WIDTH as _,
        window_height: WINDOW_HEIGHT as _,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
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

    let mut state = MainState {
        game_width: WINDOW_WIDTH as f32 * (480.0 / WINDOW_HEIGHT as f32),
        game_height: 480.0,
        camera: Vec2::ZERO,
        camera_prev: Vec2::ZERO,
        mouse: Vec2::ZERO,
        mouse_prev: Vec2::ZERO,
        zoom: 0.0,
        mouse_over_ui: false,
    };

    AnimData::initialize(&mut filesystem);
    Soldier::initialize(&mut filesystem, &config);

    let mut soldier = Soldier::new(&map.spawnpoints[0], &config);
    state.camera = soldier.particle.pos;

    let emitter: Vec<EmitterItem> = Vec::new();

    // setup window, renderer & main loop

    let mq::InternalGlContext {
        quad_context: ctx, ..
    } = unsafe { mq::get_internal_gl() };
    ctx.show_mouse(false);
    ctx.set_cursor_grab(true);
    mq::clear_background(mq::BLACK);

    let mut graphics = GameGraphics::new();
    graphics.load_sprites(&mut filesystem);
    graphics.load_map(&mut filesystem, &map);

    let time_start = instant::now();
    let current_time = || (instant::now() - time_start) / 1000.;

    let mut timecur: f64 = current_time();
    let mut timeprv: f64 = timecur;
    let mut timeacc: f64 = TIMESTEP_RATE;

    let mut zoomin_pressed;
    let mut zoomout_pressed;

    let weapons: Vec<Weapon> = WeaponKind::values()
        .iter()
        .map(|k| Weapon::new(*k, false))
        .collect();

    let bullets: Vec<Bullet> = Vec::new();

    let mut client = orb::client::Client::<MyWorld>::new(timecur, config.net.orb.clone());

    let mut world = World::new();

    let mut resources = Resources::new();
    resources.insert(map);
    resources.insert(config);
    resources.insert(state);
    resources.insert(emitter);
    resources.insert(weapons);
    resources.insert(bullets);

    physics::init(&mut world, &mut resources);
    physics::create_map_colliders(&mut world, &resources);

    let resources = resources; // This shadows the mutable binding with an immutable one.

    let mut running = true;
    while running {
        physics::systems::attach_bodies_and_colliders(&mut world);
        // physics::systems::create_joints_system();
        physics::systems::finalize_collider_attach_to_bodies(&mut world, &resources);

        networking.tick += 1;
        networking.update();

        {
            let mut state = resources.get_mut::<MainState>().unwrap();

            //             WindowEvent::CloseRequested => running = false,

            if mq::is_key_pressed(mq::KeyCode::Escape) {
                running = false;
            }
            zoomin_pressed = mq::is_key_down(mq::KeyCode::Equal);
            zoomout_pressed = mq::is_key_down(mq::KeyCode::Minus);
            if mq::is_key_pressed(mq::KeyCode::Tab) {
                let index = soldier.primary_weapon().kind.index();
                let index = (index + 1) % (WeaponKind::NoWeapon.index() + 1);
                soldier.weapons[soldier.active_weapon] =
                    resources.get_mut::<Vec<Weapon>>().unwrap()[index];
            }
            if !state.mouse_over_ui {
                soldier.update_keys();
                soldier.update_mouse_button();
            }

            let (mouse_x, mouse_y) = mq::mouse_position();
            state.mouse.x = mouse_x * state.game_width / WINDOW_WIDTH as f32;
            state.mouse.y = mouse_y * state.game_height / WINDOW_HEIGHT as f32;
        }

        timecur = current_time();
        timeacc += timecur - timeprv;
        timeprv = timecur;

        while timeacc >= TIMESTEP_RATE {
            timeacc -= TIMESTEP_RATE;

            physics::systems::step_world(&mut world, &resources);

            {
                // remove inactive bullets
                let mut bullets = resources.get_mut::<Vec<Bullet>>().unwrap();
                let mut i = 0;
                while i < bullets.len() {
                    if !bullets[i].active {
                        bullets.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            }

            // update soldiers
            soldier.update(&resources);

            {
                let mut bullets = resources.get_mut::<Vec<Bullet>>().unwrap();

                // update bullets
                for bullet in bullets.iter_mut() {
                    bullet.update(&resources);
                }

                // create emitted objects
                for item in resources.get_mut::<Vec<EmitterItem>>().unwrap().drain(..) {
                    match item {
                        EmitterItem::Bullet(params) => {
                            bullets.push(Bullet::new(&params, &*resources.get::<Config>().unwrap()))
                        }
                    };
                }
            }

            {
                // update camera
                let mut state = resources.get_mut::<MainState>().unwrap();

                state.camera_prev = state.camera;
                state.mouse_prev = state.mouse;

                if zoomin_pressed ^ zoomout_pressed {
                    state.zoom += iif!(zoomin_pressed, -1.0, 1.0) * TIMESTEP_RATE as f32;
                }

                state.camera = {
                    let z = f32::exp(state.zoom);
                    let mut m = Vec2::ZERO;

                    m.x = z * (state.mouse.x - state.game_width / 2.0) / 7.0
                        * ((2.0 * 640.0 / state.game_width - 1.0)
                            + (state.game_width - 640.0) / state.game_width * 0.0 / 6.8);
                    m.y = z * (state.mouse.y - state.game_height / 2.0) / 7.0;

                    let mut cam_v = state.camera;
                    let p = soldier.particle.pos;
                    let norm = p - cam_v;
                    let s = norm * 0.14;
                    cam_v += s;
                    cam_v += m;
                    cam_v
                };
            }

            // systems::rotate_balls(&mut world, timecur);

            timecur = current_time();
            timeacc += timecur - timeprv;
            timeprv = timecur;
        }

        let p = f64::min(1.0, f64::max(0.0, timeacc / TIMESTEP_RATE));

        graphics.render_frame(
            &world,
            &resources,
            &soldier,
            timecur - TIMESTEP_RATE * (1.0 - p),
            p as f32,
        );

        if cfg!(debug_assertions) {
            debug::build_ui(&mut world, &resources, timecur as u32, p as f32);
        }

        {
            let mut state = resources.get_mut::<MainState>().unwrap();
            let mouse_over_ui =
                macroquad::ui::root_ui().is_mouse_over(Vec2::from(mq::mouse_position()));
            if state.mouse_over_ui != mouse_over_ui {
                state.mouse_over_ui = mouse_over_ui;
                ctx.show_mouse(state.mouse_over_ui);
            }
        }

        networking.set_input_state(&soldier.control);

        networking.process(&mut *resources.get_mut::<Config>().unwrap(), &mut client);
        log::trace!("ready_client_display_state: {:?}", client.display_state());
        client.update(timeacc, timecur);
        networking.post_process(&*resources.get::<Config>().unwrap());

        physics::despawn_outliers(&mut world, &resources);
        physics::systems::collect_removals(&mut world, &resources);

        macroquad::window::next_frame().await;
        world.clear_trackers();
    }
}
