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
mod mapfile;
mod particles;
mod render;
mod soldier;
mod state;
mod weapons;

use anims::*;
use bullet::*;
use calc::*;
use control::*;
use mapfile::*;
use particles::*;
use render::*;
use soldier::*;
use state::*;
use weapons::*;

use gfx2d::macroquad::{self as macroquad, logging as log, prelude as mq};
use gvfs::filesystem::{File, Filesystem};
use megaui_macroquad::{draw_megaui, draw_window, megaui::hash, mouse_over_ui, WindowParams};
use std::{env, path};

const W: u32 = 1280;
const H: u32 = 720;

fn config() -> mq::Conf {
    mq::Conf {
        sample_count: 4,
        window_title: clap::crate_name!().to_string(),
        window_width: W as _,
        window_height: H as _,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let cmd = clap::app_from_crate!()
        .arg(
            clap::Arg::with_name("map")
                .help("name of map to load")
                .short("m")
                .long("map")
                .takes_value(true),
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
        mods.push(filesystem.open(soldat_smod).unwrap());
    }

    for f in filesystem.read_dir(path::Path::new("/")).unwrap() {
        let f = f.as_path();
        if let Some(name) = f.to_str() {
            if filesystem.is_file(f) && f != soldat_smod && name.ends_with(".smod") {
                mods.push(filesystem.open(f).unwrap());
            }
        }
    }
    for m in mods.drain(..) {
        match m {
            File::VfsFile(file) => {
                filesystem.add_zip_file(file).unwrap();
            }
        }
    }

    AnimData::initialize(&mut filesystem);
    Soldier::initialize(&mut filesystem);

    let mut map_name = cmd.value_of("map").unwrap_or("ctf_Ash").to_owned();
    map_name.push_str(".pms");

    let map = MapFile::load_map_file(&mut filesystem, map_name.as_str());
    log::info!("Using map: {}", map.mapname);

    let mut state = MainState {
        map,
        game_width: W as f32 * (480.0 / H as f32),
        game_height: 480.0,
        camera: Vec2::zero(),
        camera_prev: Vec2::zero(),
        mouse: Vec2::zero(),
        mouse_prev: Vec2::zero(),
        gravity: constants::GRAV,
        zoom: 0.0,
        bullets: vec![],
        mouse_over_ui: false,
    };

    let mut soldier = Soldier::new(&state.map.spawnpoints[0]);
    state.camera = soldier.particle.pos;

    let mut emitter: Vec<EmitterItem> = Vec::new();

    // setup window, renderer & main loop

    let mq::InternalGlContext {
        quad_context: ctx, ..
    } = unsafe { mq::get_internal_gl() };
    ctx.show_mouse(false);
    ctx.set_cursor_grab(true);
    mq::clear_background(mq::BLACK);

    let mut graphics = GameGraphics::new();
    graphics.load_sprites(&mut filesystem);
    graphics.load_map(&mut filesystem, &state.map);

    let time_start = time::precise_time_s();
    let current_time = || time::precise_time_s() - time_start;

    let mut timecur: f64 = current_time();
    let mut timeprv: f64 = timecur;
    let mut timeacc: f64 = 0.0;
    let mut running = true;

    let mut zoomin_pressed;
    let mut zoomout_pressed;

    let weapons: Vec<Weapon> = WeaponKind::values()
        .iter()
        .map(|k| Weapon::new(*k, false))
        .collect();

    while running {
        //             WindowEvent::CloseRequested => running = false,

        if mq::is_key_pressed(mq::KeyCode::Escape) {
            running = false;
        }
        zoomin_pressed = mq::is_key_down(mq::KeyCode::Equal);
        zoomout_pressed = mq::is_key_down(mq::KeyCode::Minus);
        if mq::is_key_pressed(mq::KeyCode::Tab) {
            let index = soldier.primary_weapon().kind.index();
            let index = (index + 1) % (WeaponKind::NoWeapon.index() + 1);
            soldier.weapons[soldier.active_weapon] = weapons[index];
        }
        if !state.mouse_over_ui {
            soldier.update_keys();
            soldier.update_mouse_button();
        }

        let (mouse_x, mouse_y) = mq::mouse_position();
        state.mouse.x = mouse_x * state.game_width / W as f32;
        state.mouse.y = mouse_y * state.game_height / H as f32;

        let dt = 1.0 / 60.0;

        timecur = current_time();
        timeacc += timecur - timeprv;
        timeprv = timecur;

        while timeacc >= dt {
            timeacc -= dt;

            // remove inactive bullets

            let mut i = 0;
            while i < state.bullets.len() {
                if !state.bullets[i].active {
                    state.bullets.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            // update soldiers

            soldier.update(&state, &mut emitter);

            // update bullets

            for bullet in state.bullets.iter_mut() {
                bullet.update(&state.map);
            }

            // create emitted objects

            for item in emitter.drain(..) {
                match item {
                    EmitterItem::Bullet(params) => state.bullets.push(Bullet::new(&params)),
                };
            }

            // update camera

            state.camera_prev = state.camera;
            state.mouse_prev = state.mouse;

            if zoomin_pressed ^ zoomout_pressed {
                state.zoom += iif!(zoomin_pressed, -1.0, 1.0) * dt as f32;
            }

            state.camera = {
                let z = f32::exp(state.zoom);
                let mut m = Vec2::zero();

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

            timecur = current_time();
            timeacc += timecur - timeprv;
            timeprv = timecur;
        }

        let p = f64::min(1.0, f64::max(0.0, timeacc / dt));

        graphics.render_frame(&state, &soldier, timecur - dt * (1.0 - p), p as f32);

        draw_window(
            hash!(),
            vec2(10., 10.),
            vec2(50., 20.),
            WindowParams {
                titlebar: false,
                ..Default::default()
            },
            |ui| {
                ui.label(None, "Hello!");
            },
        );

        draw_megaui();

        let mouse_over_ui = mouse_over_ui();
        if state.mouse_over_ui != mouse_over_ui {
            state.mouse_over_ui = mouse_over_ui;
            ctx.show_mouse(state.mouse_over_ui);
        }

        macroquad::window::next_frame().await
    }
}
