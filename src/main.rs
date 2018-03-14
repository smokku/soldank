extern crate gfx2d;
extern crate byteorder;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;
extern crate nalgebra as na;

use shared::anims::Animation;
use shared::parts::ParticleSystem;
use shared::mapfile::MapFile;
use shared::state::*;
use shared::soldier::*;

mod shared;
use na::Vector2;
use shared::renderer::*;

const GRAV: f32 = 0.06;

fn main() {
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

    let map = MapFile::load_map_file(&String::from("ctf_Ash.pms"));

    let mut state = MainState {
        map: map,
        anims: anims,
        soldier_parts: soldier_parts,
        gostek_skeleton: gostek,
        game_width: 768,
        game_height: 480,
        camera: Vector2::new(0.0f32, 0.0f32),
        camera_prev: Vector2::new(0.0f32, 0.0f32),
        mouse: Vector2::new(0.0f32, 0.0f32),
        mouse_prev: Vector2::new(0.0f32, 0.0f32),
        gravity: GRAV,
    };

    let mut soldier = Soldier::new(&mut state);
    // setup rendering & gameloop
    renderer::render(&mut state, &mut soldier);
}
