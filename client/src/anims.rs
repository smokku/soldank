use super::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

static mut ANIMATIONS: Option<Vec<AnimData>> = None;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Anim {
    Stand,
    Run,
    RunBack,
    Jump,
    JumpSide,
    Fall,
    Crouch,
    CrouchRun,
    Reload,
    Throw,
    Recoil,
    SmallRecoil,
    Shotgun,
    ClipOut,
    ClipIn,
    SlideBack,
    Change,
    ThrowWeapon,
    WeaponNone,
    Punch,
    ReloadBow,
    Barret,
    Roll,
    RollBack,
    CrouchRunBack,
    Cigar,
    Match,
    Smoke,
    Wipe,
    Groin,
    Piss,
    Mercy,
    Mercy2,
    TakeOff,
    Prone,
    Victory,
    Aim,
    HandsUpAim,
    ProneMove,
    GetUp,
    AimRecoil,
    HandsUpRecoil,
    Melee,
    Own,
}

#[derive(Debug)]
pub struct AnimFrame {
    pub positions: Vec<Vec2>,
}

#[derive(Debug)]
pub struct AnimData {
    pub id: Anim,
    pub looped: bool,
    pub speed: i32,
    pub frames: Vec<AnimFrame>,
}

#[derive(Debug, Copy, Clone)]
pub struct AnimState {
    pub id: Anim,
    pub looped: bool,
    pub speed: i32,
    pub count: i32,
    pub frame: usize,
}

impl Anim {
    pub fn data(&self) -> &AnimData {
        unsafe { &ANIMATIONS.as_ref().unwrap()[*self as usize] }
    }

    pub fn num_frames(&self) -> usize {
        self.data().frames.len()
    }
}

impl AnimState {
    pub fn new(id: Anim) -> AnimState {
        AnimState {
            id,
            looped: id.data().looped,
            speed: id.data().speed,
            count: 0,
            frame: 1,
        }
    }

    pub fn do_animation(&mut self) {
        self.count += 1;

        if self.count == self.speed {
            self.count = 0;
            self.frame += 1;

            if self.frame > self.num_frames() {
                if self.looped {
                    self.frame = 1;
                } else {
                    self.frame = self.num_frames();
                }
            }
        }
    }

    pub fn pos(&self, index: usize) -> Vec2 {
        self.id.data().frames[self.frame - 1].positions[index - 1]
    }

    pub fn num_frames(&self) -> usize {
        self.id.num_frames()
    }

    pub fn is_any(&self, animations: &[Anim]) -> bool {
        animations.contains(&self.id)
    }
}

impl AnimData {
    pub fn initialize(fs: &mut Filesystem) {
        unsafe {
            ANIMATIONS.replace(load_animations(fs));
        }
    }

    pub fn load_from_file(
        id: Anim,
        fs: &mut Filesystem,
        file_name: &str,
        speed: i32,
        looped: bool,
    ) -> AnimData {
        let mut path = PathBuf::from("anims/");
        path.push(file_name);

        let file = fs
            .open(&path)
            .expect(format!("Error opening animation file: {:?}", path).as_str());
        let mut line = String::new();
        let mut buf = BufReader::new(file);
        let mut frames: Vec<AnimFrame> = Vec::new();
        let mut positions: Vec<Vec2> = Vec::new();

        let add_frame = |frames: &mut Vec<AnimFrame>, positions: &[Vec2]| {
            let n = frames
                .last()
                .map_or(positions.len(), |frame| frame.positions.len());

            if positions.len() != n {
                panic!("Wrong number of points in animation frame.");
            }

            frames.push(AnimFrame {
                positions: positions.to_vec(),
            });
        };

        buf.read_line(&mut line).ok();

        while !line.is_empty() && line.trim() != "ENDFILE" {
            if line.trim() == "NEXTFRAME" {
                add_frame(&mut frames, &positions);
                positions.clear();
            } else {
                let mut coords = [0f32; 3];
                let point: usize = line.trim().parse().unwrap();

                assert!(point == positions.len() + 1);

                for coord in coords.iter_mut() {
                    line.clear();
                    buf.read_line(&mut line).ok();
                    *coord = line.trim().parse().unwrap();
                }

                positions.push(vec2(-3.0 * coords[0] / 1.1, -3.0 * coords[2]));
            }

            line.clear();
            buf.read_line(&mut line).ok();
        }

        add_frame(&mut frames, &positions);

        AnimData {
            id,
            looped,
            speed,
            frames,
        }
    }
}

fn load_animations(fs: &mut Filesystem) -> Vec<AnimData> {
    let data = [
        (Anim::Stand, "stoi.poa", 3, true),
        (Anim::Run, "biega.poa", 1, true),
        (Anim::RunBack, "biegatyl.poa", 1, true),
        (Anim::Jump, "skok.poa", 1, false),
        (Anim::JumpSide, "skokwbok.poa", 1, false),
        (Anim::Fall, "spada.poa", 1, false),
        (Anim::Crouch, "kuca.poa", 1, false),
        (Anim::CrouchRun, "kucaidzie.poa", 2, true),
        (Anim::Reload, "laduje.poa", 2, false),
        (Anim::Throw, "rzuca.poa", 1, false),
        (Anim::Recoil, "odrzut.poa", 1, false),
        (Anim::SmallRecoil, "odrzut2.poa", 1, false),
        (Anim::Shotgun, "shotgun.poa", 1, false),
        (Anim::ClipOut, "clipout.poa", 3, false),
        (Anim::ClipIn, "clipin.poa", 3, false),
        (Anim::SlideBack, "slideback.poa", 2, false),
        (Anim::Change, "change.poa", 1, false),
        (Anim::ThrowWeapon, "wyrzuca.poa", 1, false),
        (Anim::WeaponNone, "bezbroni.poa", 3, false),
        (Anim::Punch, "bije.poa", 1, false),
        (Anim::ReloadBow, "strzala.poa", 1, false),
        (Anim::Barret, "barret.poa", 9, false),
        (Anim::Roll, "skokdolobrot.poa", 1, false),
        (Anim::RollBack, "skokdolobrottyl.poa", 1, false),
        (Anim::CrouchRunBack, "kucaidzietyl.poa", 2, true),
        (Anim::Cigar, "cigar.poa", 3, false),
        (Anim::Match, "match.poa", 3, false),
        (Anim::Smoke, "smoke.poa", 4, false),
        (Anim::Wipe, "wipe.poa", 4, false),
        (Anim::Groin, "krocze.poa", 2, false),
        (Anim::Piss, "szcza.poa", 8, false),
        (Anim::Mercy, "samo.poa", 3, false),
        (Anim::Mercy2, "samo2.poa", 3, false),
        (Anim::TakeOff, "takeoff.poa", 2, false),
        (Anim::Prone, "lezy.poa", 1, false),
        (Anim::Victory, "cieszy.poa", 3, false),
        (Anim::Aim, "celuje.poa", 2, false),
        (Anim::HandsUpAim, "gora.poa", 2, false),
        (Anim::ProneMove, "lezyidzie.poa", 2, true),
        (Anim::GetUp, "wstaje.poa", 1, false),
        (Anim::AimRecoil, "celujeodrzut.poa", 1, false),
        (Anim::HandsUpRecoil, "goraodrzut.poa", 1, false),
        (Anim::Melee, "kolba.poa", 1, false),
        (Anim::Own, "rucha.poa", 3, false),
    ];

    let mut animations: Vec<AnimData> = data
        .iter()
        .map(|params| AnimData::load_from_file(params.0, fs, params.1, params.2, params.3))
        .collect();

    animations.sort_by(|a, b| (a.id as usize).cmp(&(b.id as usize)));
    animations
}
