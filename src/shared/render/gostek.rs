use super::*;
use shared::soldier::Soldier;
use ini::Ini;
use bit_array::BitArray;
use typenum::U256;
use std::str::FromStr;

type BitSet = BitArray<u64, U256>;

#[derive(Debug, Copy, Clone)]
pub enum GostekSprite {
    None,
    Gostek(Gostek),
    Weapon(Weapon),
}

impl GostekSprite {
    pub fn is_none(&self) -> bool {
        match *self {
            GostekSprite::None => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GostekColor {
    None,
    Main,
    Pants,
    Skin,
    Hair,
    Cygar,
    Headblood,
}

#[derive(Debug, Copy, Clone)]
pub enum GostekAlpha {
    Base,
    Blood,
    Nades,
}

#[derive(Debug, Copy, Clone)]
pub struct GostekPartInfo {
    pub name: &'static str,
    pub sprite: GostekSprite,
    pub point: (usize, usize),
    pub center: (f32, f32),
    pub flexibility: f32,
    pub flip: bool,
    pub team: bool,
    pub color: GostekColor,
    pub alpha: GostekAlpha,
    pub visible: bool,
}

pub struct GostekGraphics {
    pub data: Vec<GostekPartInfo>,
    pub base_visibility: BitSet,
}

impl GostekGraphics {
    pub fn new() -> GostekGraphics {
        GostekGraphics {
            data: GostekPart::data().to_vec(),
            base_visibility: BitSet::from_fn(|i| GostekPart::data().get(i).map_or(false, |p| p.visible)),
        }
    }

    pub fn load_data(&mut self, cfg: &Ini) {
        self.data = GostekPart::data().to_vec();

        if let Some(data) = cfg.section(Some("GOSTEK".to_owned())) {
            let mut key = String::with_capacity(256);

            let copy_and_insert_underscores = |dest: &mut String, source: &str| {
                for (i, ch) in source.chars().enumerate() {
                    if i > 0 && ch.is_uppercase() { dest.push('_') };
                    dest.push(ch);
                }
            };

            for part in &mut self.data {
                key.clear();
                copy_and_insert_underscores(&mut key, part.name);

                let len = key.len();
                key.push_str("_CenterX");

                if let Some(value) = data.get(&key) {
                    part.center.0 = f32::from_str(value).unwrap_or(part.center.0);
                }

                key.truncate(len);
                key.push_str("_CenterY");

                if let Some(value) = data.get(&key) {
                    part.center.1 = f32::from_str(value).unwrap_or(part.center.1);
                }
            }
        }
    }

    pub fn render(&self, soldier: &Soldier, batch: &mut DrawBatch, sprites: &Vec<Vec<Sprite>>) {
        let mut visible = self.base_visibility.clone();
        let mut alpha_base = soldier.alpha;
        let mut alpha_blood = f32::max(0.0, f32::min(255.0, 200.0 - soldier.health.round())) as u8;
        let mut color_cygar = rgb(255, 255, 255);
        let color_none      = rgb(255, 255, 255);
        let color_main      = rgb( 0 ,  0 ,  0 ); // TODO: Player.Color1
        let color_pants     = rgb( 0 ,  0 ,  0 ); // TODO: Player.Color2
        let color_skin      = rgb(230, 180, 120); // TODO: Player.SkinColor
        let color_hair      = rgb( 0 ,  0 ,  0 ); // TODO: Player.HairColor
        let color_headblood = rgb(172, 169, 168);
        let alpha_nades: u8;

        if soldier.has_cigar == 5 {
            color_cygar = rgb(97, 97, 97);
        }

        let realistic_mode = false;

        if soldier.health > (90.0 - 40.0 * f32::from(realistic_mode as u8)) {
            alpha_blood = 0;
        }

        if realistic_mode && soldier.visible > 0 && soldier.visible < 45 && soldier.alpha > 60 {
            // TODO: if this really needs to change it should be done somewhere during update, not here
            // soldier.alpha = 3 * soldier.visible;
            alpha_base = 3 * soldier.visible;
            alpha_blood = 0;
        }

        alpha_nades = (0.75 * (alpha_base as f32)) as u8;

        if alpha_blood > 0 {
            visible.set(GostekPart::LeftThighDmg.id(), true);
            visible.set(GostekPart::LeftLowerlegDmg.id(), true);
            visible.set(GostekPart::LeftForearmDmg.id(), true);
            visible.set(GostekPart::LeftArmDmg.id(), true);
            visible.set(GostekPart::ChestDmg.id(), true);
            visible.set(GostekPart::HipDmg.id(), true);
            visible.set(GostekPart::HeadDmg.id(), true);
            visible.set(GostekPart::RightThighDmg.id(), true);
            visible.set(GostekPart::RightLowerlegDmg.id(), true);
            visible.set(GostekPart::RightForearmDmg.id(), true);
            visible.set(GostekPart::RightArmDmg.id(), true);
        }

        if soldier.control.jets && soldier.jets_count > 0 {
            visible.set(GostekPart::LeftFoot.id(), false);
            visible.set(GostekPart::RightFoot.id(), false);
            visible.set(GostekPart::LeftJetfoot.id(), true);
            visible.set(GostekPart::RightJetfoot.id(), true);
        }

        if soldier.vest > 0.0 {
            visible.set(GostekPart::Vest.id(), true);
        }

        // TODO:    if soldier.tertiary_weapon.num == guns[frag_grenade].num
        let index = if true {
            GostekPart::FragGrenade1.id()
        } else {
            GostekPart::ClusterGrenade1.id()
        };

        // TODO: use actual values
        const THROW_ANIM: i32 = 9;
        let tertiary_ammo_count = 3;
        let n = i32::min(5, tertiary_ammo_count - iif!(soldier.body_animation.id == THROW_ANIM, 1, 0));

        for i in 0..n {
            visible.set(index + i as usize, true);
        }

        let chain = 0; // TODO: Player.Chain (this seems broken, check skeleton)

        match chain {
            1 => {
                visible.set(GostekPart::SilverLchain.id(), true);
                visible.set(GostekPart::SilverRchain.id(), true);
                visible.set(GostekPart::SilverPendant.id(), true);
            },
            2 => {
                visible.set(GostekPart::GoldenLchain.id(), true);
                visible.set(GostekPart::GoldenRchain.id(), true);
                visible.set(GostekPart::GoldenPendant.id(), true);
            },
            _ => {}
        }

        if soldier.has_cigar == 5 || soldier.has_cigar == 10 {
            visible.set(GostekPart::Cigar.id(), true);
        }

        if soldier.dead_meat {
            visible.set(GostekPart::Head.id(), false);
            visible.set(GostekPart::HeadDmg.id(), false);
            visible.set(GostekPart::HeadDead.id(), true);
            visible.set(GostekPart::HeadDeadDmg.id(), true);
        }

        // TODO: if weapon is bow or fire bow
        if false {
            visible.set(GostekPart::RamboBadge.id(), true);
        } else {
            const ANIM_WIPE: i32 = 28;
            const ANIM_TAKEOFF: i32 = 33;

            let grabbed = match soldier.body_animation.id {
                ANIM_WIPE | ANIM_TAKEOFF => soldier.body_animation.curr_frame > 4,
                _ => false,
            };

            if soldier.wear_helmet == 1 {
                let head_cap = Gostek::Helm; // TODO: Player.HeadCap

                match head_cap {
                    Gostek::Helm => match grabbed {
                        true => visible.set(GostekPart::GrabbedHelmet.id(), true),
                        false => visible.set(GostekPart::Helmet.id(), true),
                    },
                    Gostek::Kap => match grabbed {
                        true => visible.set(GostekPart::GrabbedHat.id(), true),
                        false => visible.set(GostekPart::Hat.id(), true),
                    },
                    _ => {}
                }
            }

            let hair_style = 3; // TODO: Player.HairStyle

            if grabbed || soldier.wear_helmet != 1 || hair_style == 3 {
                match hair_style {
                    1 => for i in 0..6 { visible.set(GostekPart::HairDreadlocks.id() + i, true); },
                    2 => visible.set(GostekPart::HairPunk.id(), true),
                    3 => visible.set(GostekPart::MrT.id(), true),
                    4 => visible.set(GostekPart::HairNormal.id(), true),
                    _ => {},
                }
            }
        }

        // TODO: primary/secondary weapons

        for (i, part) in self.data.iter().enumerate() {
            if visible[i] && !part.sprite.is_none() {
                let mut sid: usize = 0;
                let mut cx = part.center.0;
                let mut cy = part.center.1;
                let mut scale = vec2(1.0, 1.0);
                let p0 = soldier.skeleton.pos[part.point.0];
                let p1 = soldier.skeleton.pos[part.point.1];
                let rot = f32::atan2(p1.y - p0.y, p1.x - p0.x);

                if soldier.direction != 1 {
                    match part.flip {
                        true =>  { cy = 1.0 - cy; sid += 1; },
                        false => { scale.y = -1.0; },
                    }
                }

                if part.flexibility > 0.0 {
                    scale.x = f32::min(1.5, f32::sqrt((p1.x - p0.x).powi(2) + (p1.y - p0.y).powi(2)) / part.flexibility);
                }

                let mut color = match part.color {
                    GostekColor::None      => color_none,
                    GostekColor::Main      => color_main,
                    GostekColor::Pants     => color_pants,
                    GostekColor::Skin      => color_skin,
                    GostekColor::Hair      => color_hair,
                    GostekColor::Cygar     => color_cygar,
                    GostekColor::Headblood => color_headblood,
                };

                match part.alpha {
                    GostekAlpha::Base  => color.set_a(alpha_base),
                    GostekAlpha::Blood => color.set_a(alpha_blood),
                    GostekAlpha::Nades => color.set_a(alpha_nades),
                };

                match part.sprite {
                    GostekSprite::Gostek(spriteid) => {
                        let spriteid = spriteid + sid;
                        let sprite = &sprites[spriteid.group().id()][spriteid.id()];
                        let (w, h) = (sprite.width, sprite.height);

                        batch.add_tinted_sprite(sprite, color, Transform::WithPivot {
                            pivot: vec2(cx * w, cy * h),
                            pos: vec2(p0.x, p0.y + 1.0),
                            scale,
                            rot,
                        });
                    },
                    _ => {}
                }
            }
        }
    }
}
