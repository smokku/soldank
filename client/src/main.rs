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
mod game;
mod mapfile;
mod networking;
mod particles;
mod physics;
mod render;
mod soldier;
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

use game::components::{EmitterItem, Team};
use soldank_shared::{networking::GameWorld, orb};

fn main() {
    color_eyre::install().unwrap();
    engine::Logger::init();

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
                .allow_hyphen_values(true)
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

    // let mut networking = Networking::new(cmd.value_of("connect"));
    // if let Some(key) = cmd.value_of("key") {
    //     networking.connection_key = key.to_string();
    // }
    // if let Some(nick) = cmd.value_of("nick") {
    //     networking.nick_name = nick.to_string();
    // }

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

    AnimData::initialize(&mut filesystem);
    Soldier::initialize(&mut filesystem, &config);

    let weapons: Vec<Weapon> = WeaponKind::values()
        .iter()
        .map(|k| Weapon::new(*k, false))
        .collect();

    // let client = orb::client::Client::<GameWorld>::new(mq::date::now(), config.net.orb.clone());

    let mut world = World::new();

    let mut resources = Resources::new();

    resources.insert(map);
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
    resources.insert(physics::JointsEntityMap::default());
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

// fn update(&mut self, _ctx: &mut mq::Context) {
//     self.networking.tick += 1;
//     self.networking.update();

//     while self.timeacc >= TIMESTEP_RATE {
//         self.timeacc -= TIMESTEP_RATE;

//         // remove inactive bullets
//         let mut i = 0;
//         while i < self.bullets.len() {
//             if !self.bullets[i].active {
//                 self.bullets.swap_remove(i);
//             } else {
//                 i += 1;
//             }
//         }

//         // // update soldiers

//         // update bullets
//         for bullet in self.bullets.iter_mut() {
//             bullet.update(&self.resources);
//         }

//         // // create emitted objects
//         // for item in self.emitter.drain(..) {
//         //     match item {
//         //         EmitterItem::Bullet(params) => self.bullets.push(Bullet::new(
//         //             &params,
//         //             &*self.resources.get::<Config>().unwrap(),
//         //         )),
//         //     };
//         // }
//     }

//     self.networking.set_input_state(&self.soldier.control);

//     self.networking.process(&self.resources, &mut self.client);
//     self.client.update(self.timeacc, self.last_frame);
//     if let Some(state) = self.client.display_state() {
//         log::trace!("client_display_state: {}", state.display_state().len());
//     }
//     self.networking
//         .post_process(&*self.resources.get::<Config>().unwrap());
// }
