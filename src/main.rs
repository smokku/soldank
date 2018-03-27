extern crate glutin;
extern crate gfx2d;
extern crate byteorder;
extern crate time;
extern crate nalgebra as na;
extern crate ini;
extern crate typenum;
extern crate bit_array;
extern crate clap;

macro_rules! iif(($cond:expr, $then:expr, $otherwise:expr) => (if $cond { $then } else { $otherwise }));

use na::Vector2;
use glutin::*;

use shared::calc::*;
use shared::anims::Animation;
use shared::parts::ParticleSystem;
use shared::mapfile::MapFile;
use shared::state::*;
use shared::soldier::*;
use shared::render::*;
use clap::{App, Arg};

mod shared;

const GRAV: f32 = 0.06;

fn main() {

    let cmd = App::new("Soldank")
                .about("open source clone of Soldat engine written in rust")
                .version("0.0.1")
                .arg(Arg::with_name("map")
                    .help("name of map to load")
                    .short("m")
                    .long("map")
                    .takes_value(true))
                .get_matches();

    let anims = AnimsList {
        stand: Animation::load_from_file(&String::from("stoi.poa"), 0, 3, true),
        run: Animation::load_from_file(&String::from("biega.poa"), 1, 1, true),
        run_back: Animation::load_from_file(&String::from("biegatyl.poa"), 2, 1, true),
        jump: Animation::load_from_file(&String::from("skok.poa"), 3, 1, false),
        jump_side: Animation::load_from_file(&String::from("skokwbok.poa"), 4, 1, false),
        fall: Animation::load_from_file(&String::from("spada.poa"), 5, 1, false),
        crouch: Animation::load_from_file(&String::from("kuca.poa"), 6, 1, false),
        crouch_run: Animation::load_from_file(&String::from("kucaidzie.poa"), 7, 2, true),
        reload: Animation::load_from_file(&String::from("laduje.poa"), 8, 2, false),
        throw: Animation::load_from_file(&String::from("rzuca.poa"), 9, 1, false),
        recoil: Animation::load_from_file(&String::from("odrzut.poa"), 10, 1, false),
        small_recoil: Animation::load_from_file(&String::from("odrzut2.poa"), 11, 1, false),
        shotgun: Animation::load_from_file(&String::from("shotgun.poa"), 12, 1, false),
        clip_out: Animation::load_from_file(&String::from("clipout.poa"), 13, 3, false),
        clip_in: Animation::load_from_file(&String::from("clipin.poa"), 14, 3, false),
        slide_back: Animation::load_from_file(&String::from("slideback.poa"), 15, 2, false),
        change: Animation::load_from_file(&String::from("change.poa"), 16, 0, false),
        throw_weapon: Animation::load_from_file(&String::from("wyrzuca.poa"), 17, 1, false),
        weapon_none: Animation::load_from_file(&String::from("bezbroni.poa"), 18, 3, false),
        punch: Animation::load_from_file(&String::from("bije.poa"), 19, 0, false),
        reload_bow: Animation::load_from_file(&String::from("strzala.poa"), 20, 1, false),
        barret: Animation::load_from_file(&String::from("barret.poa"), 21, 9, false),
        roll: Animation::load_from_file(&String::from("skokdolobrot.poa"), 22, 1, false),
        roll_back: Animation::load_from_file(&String::from("skokdolobrottyl.poa"), 23, 1, false),
        crouch_run_back: Animation::load_from_file(&String::from("kucaidzietyl.poa"), 24, 2, true),
        cigar: Animation::load_from_file(&String::from("cigar.poa"), 25, 3, false),
        match_: Animation::load_from_file(&String::from("match.poa"), 26, 3, false),
        smoke: Animation::load_from_file(&String::from("smoke.poa"), 27, 4, false),
        wipe: Animation::load_from_file(&String::from("wipe.poa"), 28, 4, false),
        groin: Animation::load_from_file(&String::from("krocze.poa"), 29, 2, false),
        piss: Animation::load_from_file(&String::from("szcza.poa"), 30, 8, false),
        mercy: Animation::load_from_file(&String::from("samo.poa"), 31, 3, false),
        mercy2: Animation::load_from_file(&String::from("samo2.poa"), 32, 3, false),
        take_off: Animation::load_from_file(&String::from("takeoff.poa"), 33, 2, false),
        prone: Animation::load_from_file(&String::from("lezy.poa"), 34, 1, false),
        victory: Animation::load_from_file(&String::from("cieszy.poa"), 35, 3, false),
        aim: Animation::load_from_file(&String::from("celuje.poa"), 36, 2, false),
        hands_up_aim: Animation::load_from_file(&String::from("gora.poa"), 37, 2, false),
        prone_move: Animation::load_from_file(&String::from("lezyidzie.poa"), 38, 2, true),
        get_up: Animation::load_from_file(&String::from("wstaje.poa"), 39, 1, false),
        aim_recoil: Animation::load_from_file(&String::from("celujeodrzut.poa"), 40, 1, false),
        hands_up_recoil: Animation::load_from_file(&String::from("goraodrzut.poa"), 41, 1, false),
        melee: Animation::load_from_file(&String::from("kolba.poa"), 42, 1, false),
        own: Animation::load_from_file(&String::from("rucha.poa"), 43, 3, false),
    };
    let mut gostek = ParticleSystem::new();
    gostek.load_from_file(&String::from("gostek.po"), 4.50);
    gostek.timestep = 1.00;
    gostek.gravity = 1.06 * GRAV;
    gostek.v_damping = 0.9945;

    let mut soldier_parts = ParticleSystem::new();

    soldier_parts.timestep = 1.0;
    soldier_parts.gravity = GRAV;
    soldier_parts.e_damping = 0.99;

    let mut map_name = cmd.value_of("map").unwrap_or("ctf_Ash").to_owned();

    map_name.push_str(".pms");

    let map = MapFile::load_map_file(map_name.as_str());

    const W: u32 = 1280;
    const H: u32 = 720;

    let mut state = MainState {
        map: map,
        anims: anims,
        soldier_parts: soldier_parts,
        gostek_skeleton: gostek,
        game_width: W as f32 * (480.0 / H as f32),
        game_height: 480.0,
        camera: Vector2::new(0.0f32, 0.0f32),
        camera_prev: Vector2::new(0.0f32, 0.0f32),
        mouse: Vector2::new(0.0f32, 0.0f32),
        mouse_prev: Vector2::new(0.0f32, 0.0f32),
        gravity: GRAV,
        zoom: 0.0,
    };

    let mut soldier = Soldier::new(&mut state);
    state.camera = state.soldier_parts.pos[1];

    // setup window, renderer & main loop

    let mut context = gfx2d::Gfx2dContext::initialize("Soldank", W, H);
    context.wnd.window().set_cursor(glutin::MouseCursor::NoneCursor);
    context.wnd.window().set_cursor_state(glutin::CursorState::Grab).unwrap();
    context.clear(gfx2d::rgb(0, 0, 0));
    context.present();

    let mut graphics = GameGraphics::new(&mut context);
    graphics.load_sprites(&mut context);
    graphics.load_map(&mut context, &state.map);

    let time_start = time::precise_time_s();
    let current_time = || {time::precise_time_s() - time_start};

    let mut timecur: f64 = current_time();
    let mut timeprv: f64 = timecur;
    let mut timeacc: f64 = 0.0;
    let mut running = true;

    let mut zoomin_pressed = false;
    let mut zoomout_pressed = false;

    while running {
        context.evt.poll_events(|event| match event {
            Event::WindowEvent{event, ..} => match event {
                WindowEvent::Closed => running = false,
                WindowEvent::KeyboardInput{input, ..} => {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => running = false,
                        Some(VirtualKeyCode::Add) => zoomin_pressed = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        },
                        Some(VirtualKeyCode::Subtract) => zoomout_pressed = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        },
                        _ => soldier.update_keys(&input),
                    }
                },
                WindowEvent::MouseInput{state, button, ..} => {
                    soldier.update_mouse_button(&(state, button));
                },
                WindowEvent::CursorMoved{position: (x, y), ..} => {
                    state.mouse.x = x as f32 * state.game_width / W as f32;
                    state.mouse.y = y as f32 * state.game_height / H as f32;
                },
                _ => (),
            },
            _ => (),
        });

        let dt = 1.0/60.0;

        timecur = current_time();
        timeacc += timecur - timeprv;
        timeprv = timecur;

        while timeacc >= dt {
            timeacc -= dt;

            state.soldier_parts.do_eurler_timestep_for(1);
            soldier.update(&mut state);

            state.soldier_parts.old_pos[2] = state.soldier_parts.pos[2];
            state.soldier_parts.pos[2].x += 50.0 * dt as f32;

            state.camera_prev = state.camera;
            state.mouse_prev = state.mouse;

            if zoomin_pressed ^ zoomout_pressed {
                state.zoom += iif!(zoomin_pressed, -1.0, 1.0) * dt as f32;
            }

            state.camera = {
                let z = f32::exp(state.zoom);
                let mut m = Vec2::zeros();

                m.x = z * (state.mouse.x - state.game_width / 2.0) / 7.0
                    * ((2.0 * 640.0 / state.game_width - 1.0)
                    + (state.game_width - 640.0) / state.game_width * 0.0 / 6.8);
                m.y = z * (state.mouse.y - state.game_height / 2.0) / 7.0;

                let mut cam_v = state.camera;

                let p = vec2(state.soldier_parts.pos[1].x, state.soldier_parts.pos[1].y);
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

        let p = f64::min(1.0, f64::max(0.0, timeacc/dt));
        graphics.render_frame(&mut context, &state, &soldier, timecur - dt*(1.0 - p), p as f32);
        context.present();

        // only sleep if no vsync (or if vsync doesn't wait), also needs timeBeginPeriod(1)
        // std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
