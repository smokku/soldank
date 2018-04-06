#[macro_use]
extern crate lazy_static;
extern crate bit_array;
extern crate byteorder;
extern crate clap;
extern crate gfx2d;
extern crate glutin;
extern crate ini;
extern crate time;
extern crate typenum;

macro_rules! iif(
    ($cond:expr, $then:expr, $else:expr) => (if $cond { $then } else { $else })
);

mod anims;
mod calc;
mod control;
mod mapfile;
mod particles;
mod render;
mod soldier;
mod state;
mod weapons;

use anims::*;
use calc::*;
use control::*;
use mapfile::*;
use particles::*;
use render::*;
use soldier::*;
use state::*;
use weapons::*;

use clap::{App, Arg};
use glutin::*;

const GRAV: f32 = 0.06;

fn main() {
    let cmd = App::new("Soldank")
        .about("open source clone of Soldat engine written in rust")
        .version("0.0.1")
        .arg(
            Arg::with_name("map")
                .help("name of map to load")
                .short("m")
                .long("map")
                .takes_value(true),
        )
        .get_matches();

    AnimData::initialize();
    Soldier::initialize();

    let mut map_name = cmd.value_of("map").unwrap_or("ctf_Ash").to_owned();
    map_name.push_str(".pms");

    let map = MapFile::load_map_file(map_name.as_str());

    const W: u32 = 1280;
    const H: u32 = 720;

    let mut state = MainState {
        map,
        game_width: W as f32 * (480.0 / H as f32),
        game_height: 480.0,
        camera: Vec2::zero(),
        camera_prev: Vec2::zero(),
        mouse: Vec2::zero(),
        mouse_prev: Vec2::zero(),
        gravity: GRAV,
        zoom: 0.0,
    };

    let mut soldier = Soldier::new(&state.map.spawnpoints[0]);
    state.camera = soldier.particle.pos;

    // setup window, renderer & main loop

    let mut context = gfx2d::Gfx2dContext::initialize("Soldank", W, H);
    context
        .wnd
        .window()
        .set_cursor(glutin::MouseCursor::NoneCursor);
    context
        .wnd
        .window()
        .set_cursor_state(glutin::CursorState::Grab)
        .unwrap();
    context.clear(gfx2d::rgb(0, 0, 0));
    context.present();

    let mut graphics = GameGraphics::new(&mut context);
    graphics.load_sprites(&mut context);
    graphics.load_map(&mut context, &state.map);

    let time_start = time::precise_time_s();
    let current_time = || time::precise_time_s() - time_start;

    let mut timecur: f64 = current_time();
    let mut timeprv: f64 = timecur;
    let mut timeacc: f64 = 0.0;
    let mut running = true;

    let mut zoomin_pressed = false;
    let mut zoomout_pressed = false;

    let weapons: Vec<Weapon> = WeaponKind::values()
        .iter()
        .map(|k| Weapon::new(*k, false))
        .collect();

    while running {
        context.evt.poll_events(|e| {
            if let Event::WindowEvent { event, .. } = e {
                match event {
                    WindowEvent::Closed => running = false,
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => running = false,
                        Some(VirtualKeyCode::Add) => {
                            zoomin_pressed = match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        }
                        Some(VirtualKeyCode::Subtract) => {
                            zoomout_pressed = match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            }
                        }
                        Some(VirtualKeyCode::Tab) => {
                            if input.state == ElementState::Pressed {
                                let index = soldier.primary_weapon().kind.index();
                                let index = (index + 1) % (WeaponKind::NoWeapon.index() + 1);
                                soldier.weapons[soldier.active_weapon] = weapons[index];
                            }
                        }
                        _ => soldier.update_keys(&input),
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        soldier.update_mouse_button(&(state, button));
                    }
                    WindowEvent::CursorMoved {
                        position: (x, y), ..
                    } => {
                        state.mouse.x = x as f32 * state.game_width / W as f32;
                        state.mouse.y = y as f32 * state.game_height / H as f32;
                    }
                    _ => (),
                }
            }
        });

        let dt = 1.0 / 60.0;

        timecur = current_time();
        timeacc += timecur - timeprv;
        timeprv = timecur;

        while timeacc >= dt {
            timeacc -= dt;

            soldier.update(&state);

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
        graphics.render_frame(
            &mut context,
            &state,
            &soldier,
            timecur - dt * (1.0 - p),
            p as f32,
        );
        context.present();

        // only sleep if no vsync (or if vsync doesn't wait), also needs timeBeginPeriod(1)
        // std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
