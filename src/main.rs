#[macro_use]
extern crate lazy_static;

macro_rules! iif(
    ($cond:expr, $then:expr, $else:expr) => (if $cond { $then } else { $else })
);

mod anims;
mod bullet;
mod calc;
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

use clap::{App, Arg};
use gfx2d::mq;

const GRAV: f32 = 0.06;
const W: u32 = 1280;
const H: u32 = 720;
const DT: f64 = 1.0 / 60.0;

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

    let conf = mq::conf::Conf {
        sample_count: 4,
        window_title: clap::crate_name!().to_string(),
        window_width: W as _,
        window_height: H as _,
        ..Default::default()
    };
    mq::start(conf, |mut ctx| {
        mq::UserData::owning(GameStage::new(&mut ctx, map), ctx)
    });
}

pub struct GameStage {
    state: MainState,

    context: gfx2d::Gfx2dContext,

    graphics: GameGraphics,
    last_frame: f64,
    timeacc: f64,

    soldier: Soldier,
    emitter: Vec<EmitterItem>,
    weapons: Vec<Weapon>,

    zoomin_pressed: bool,
    zoomout_pressed: bool,
}

impl GameStage {
    pub fn new(ctx: &mut mq::Context, map: MapFile) -> Self {
        let soldier = Soldier::new(&map.spawnpoints[0]);

        let emitter = Vec::new();

        // setup window, renderer & main loop
        let context = gfx2d::Gfx2dContext::new(ctx);

        ctx.show_mouse(false);
        ctx.set_cursor_grab(true);

        let mut graphics = GameGraphics::new();
        graphics.load_sprites(ctx);
        graphics.load_map(ctx, &map);

        let weapons: Vec<Weapon> = WeaponKind::values()
            .iter()
            .map(|k| Weapon::new(*k, false))
            .collect();

        GameStage {
            state: MainState {
                map,
                game_width: W as f32 * (480.0 / H as f32),
                game_height: 480.0,
                camera: soldier.particle.pos,
                camera_prev: Vec2::zero(),
                mouse: Vec2::zero(),
                mouse_prev: Vec2::zero(),
                gravity: GRAV,
                zoom: 0.0,
                bullets: vec![],
            },

            context,

            graphics,
            last_frame: mq::date::now(),
            timeacc: 0.0,

            soldier,
            emitter,
            weapons,

            zoomin_pressed: false,
            zoomout_pressed: false,
        }
    }
}

impl mq::EventHandler for GameStage {
    fn update(&mut self, _ctx: &mut mq::Context) {
        let time = mq::date::now();
        self.timeacc += time - self.last_frame;
        self.last_frame = time;

        while self.timeacc >= DT {
            self.timeacc -= DT;

            // remove inactive bullets

            let mut i = 0;
            while i < self.state.bullets.len() {
                if !self.state.bullets[i].active {
                    self.state.bullets.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            // update soldiers

            self.soldier.update(&self.state, &mut self.emitter);

            // update bullets

            for bullet in self.state.bullets.iter_mut() {
                bullet.update(&self.state.map);
            }

            // create emitted objects

            for item in self.emitter.drain(..) {
                match item {
                    EmitterItem::Bullet(params) => self.state.bullets.push(Bullet::new(&params)),
                };
            }

            // update camera

            self.state.camera_prev = self.state.camera;
            self.state.mouse_prev = self.state.mouse;

            if self.zoomin_pressed ^ self.zoomout_pressed {
                self.state.zoom += iif!(self.zoomin_pressed, -1.0, 1.0) * DT as f32;
            }

            self.state.camera = {
                let z = f32::exp(self.state.zoom);
                let mut m = Vec2::zero();

                m.x = z * (self.state.mouse.x - self.state.game_width / 2.0) / 7.0
                    * ((2.0 * 640.0 / self.state.game_width - 1.0)
                        + (self.state.game_width - 640.0) / self.state.game_width * 0.0 / 6.8);
                m.y = z * (self.state.mouse.y - self.state.game_height / 2.0) / 7.0;

                let mut cam_v = self.state.camera;
                let p = self.soldier.particle.pos;
                let norm = p - cam_v;
                let s = norm * 0.14;
                cam_v += s;
                cam_v += m;
                cam_v
            };

            let time = mq::date::now();
            self.timeacc += time - self.last_frame;
            self.last_frame = time;
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        _keymods: mq::KeyMods,
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
                let index = self.soldier.primary_weapon().kind.index();
                let index = (index + 1) % (WeaponKind::NoWeapon.index() + 1);
                self.soldier.weapons[self.soldier.active_weapon] = self.weapons[index];
            }
            _ => self.soldier.update_keys(true, keycode),
        }
    }

    fn key_up_event(
        &mut self,
        ctx: &mut gfx2d::Context,
        keycode: mq::KeyCode,
        _keymods: mq::KeyMods,
    ) {
        match keycode {
            mq::KeyCode::Escape => ctx.request_quit(),
            mq::KeyCode::Equal => {
                self.zoomin_pressed = false;
            }
            mq::KeyCode::Minus => {
                self.zoomout_pressed = false;
            }
            _ => self.soldier.update_keys(false, keycode),
        }
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut mq::Context,
        button: mq::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.soldier.update_mouse_button(true, button);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut gfx2d::Context,
        button: mq::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.soldier.update_mouse_button(false, button);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut mq::Context, x: f32, y: f32) {
        self.state.mouse.x = x * self.state.game_width / W as f32;
        self.state.mouse.y = y * self.state.game_height / H as f32;
    }

    fn draw(&mut self, ctx: &mut mq::Context) {
        let p = f64::min(1.0, f64::max(0.0, self.timeacc / DT));

        self.graphics.render_frame(
            &mut self.context,
            ctx,
            &self.state,
            &self.soldier,
            self.last_frame - DT * (1.0 - p),
            p as f32,
        );

        ctx.commit_frame();
    }
}
