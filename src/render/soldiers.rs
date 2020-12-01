use super::*;
use bit_array::BitArray;
use gfx::{SoldierPart, SpriteData};
use ini::Ini;
use std::str::FromStr;
use typenum::U256;

type BitSet = BitArray<u64, U256>;

#[derive(Debug, Copy, Clone)]
pub enum SoldierSprite {
    None,
    Soldier(gfx::Soldier),
    Weapon(gfx::Weapon),
}

impl SoldierSprite {
    pub fn is_none(&self) -> bool {
        match *self {
            SoldierSprite::None => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SoldierColor {
    None,
    Main,
    Pants,
    Skin,
    Hair,
    Cygar,
    Headblood,
}

#[derive(Debug, Copy, Clone)]
pub enum SoldierAlpha {
    Base,
    Blood,
    Nades,
}

#[derive(Debug, Copy, Clone)]
pub struct SoldierPartInfo {
    pub name: &'static str,
    pub sprite: SoldierSprite,
    pub point: (usize, usize),
    pub center: (f32, f32),
    pub flexibility: f32,
    pub flip: bool,
    pub team: bool,
    pub color: SoldierColor,
    pub alpha: SoldierAlpha,
    pub visible: bool,
}

pub struct SoldierGraphics {
    pub parts: Vec<SoldierPartInfo>,
    pub base_visibility: BitSet,
}

impl SoldierGraphics {
    pub fn new() -> SoldierGraphics {
        SoldierGraphics {
            parts: SoldierPart::data().to_vec(),
            base_visibility: BitSet::from_fn(|i| {
                SoldierPart::data().get(i).map_or(false, |p| p.visible)
            }),
        }
    }

    pub fn load_data(&mut self, cfg: &Ini) {
        self.parts = SoldierPart::data().to_vec();

        if let Some(data) = cfg.section(Some("GOSTEK".to_owned())) {
            let mut key = String::with_capacity(256);

            let copy_and_insert_underscores = |dest: &mut String, source: &str| {
                for (i, ch) in source.chars().enumerate() {
                    if i > 0 && ch.is_uppercase() {
                        dest.push('_')
                    };
                    dest.push(ch);
                }
            };

            for part in &mut self.parts {
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
}

pub fn render_soldier(
    soldier: &Soldier,
    soldier_graphics: &SoldierGraphics,
    sprites: &[Vec<Sprite>],
    batch: &mut DrawBatch,
    frame_percent: f32,
) {
    let sk = &soldier.skeleton;
    let (colors, alpha) = colors_and_alpha(soldier);
    let has_blood = alpha[SoldierAlpha::Blood as usize] > 0;
    let visible = parts_visibility(&soldier_graphics.base_visibility, soldier, has_blood);

    // TODO: team2 offset, dredlock rotation matrix

    for (i, part) in soldier_graphics.parts.iter().enumerate() {
        if visible[i] && !part.sprite.is_none() {
            let mut sprite_index: usize = 0;
            let cx = part.center.0;
            let mut cy = part.center.1;
            let mut scale = vec2(1.0, 1.0);
            let (p0, p1) = part.point;
            let p0 = lerp(sk.old_pos(p0), sk.pos(p0), frame_percent);
            let p1 = lerp(sk.old_pos(p1), sk.pos(p1), frame_percent);
            let rot = vec2angle(p1 - p0);

            if soldier.direction != 1 {
                if part.flip {
                    cy = 1.0 - cy;
                    sprite_index += 1;
                } else {
                    scale.y = -1.0;
                }
            }

            if part.flexibility > 0.0 {
                scale.x = f32::min(1.5, (p1 - p0).length() / part.flexibility);
            }

            let color = {
                let color = colors[part.color as usize];
                rgba(color.r(), color.g(), color.b(), alpha[part.alpha as usize])
            };

            let sprite = match part.sprite {
                SoldierSprite::Soldier(part_sprite) => {
                    let group = gfx::Group::Soldier;
                    let sprite = part_sprite + sprite_index;
                    &sprites[group.id()][sprite.id()]
                }
                SoldierSprite::Weapon(part_sprite) => {
                    let group = gfx::Group::Weapon;
                    let sprite = part_sprite + sprite_index;
                    &sprites[group.id()][sprite.id()]
                }
                SoldierSprite::None => unreachable!(),
            };

            batch.add_sprite(
                sprite,
                color,
                Transform::WithPivot {
                    pivot: vec2(cx * sprite.width, cy * sprite.height),
                    pos: vec2(p0.x, p0.y + 1.0),
                    scale,
                    rot,
                },
            );
        }
    }
}

fn colors_and_alpha(soldier: &Soldier) -> ([Color; 7], [u8; 3]) {
    let mut alpha_base = soldier.alpha;
    let mut alpha_blood = f32::max(0.0, f32::min(255.0, 200.0 - soldier.health.round())) as u8;
    let mut color_cygar = rgb(255, 255, 255);
    let color_none = rgb(255, 255, 255);
    let color_main = rgb(0, 0, 0); // TODO: Player.Color1
    let color_pants = rgb(0, 0, 0); // TODO: Player.Color2
    let color_skin = rgb(230, 180, 120); // TODO: Player.SkinColor
    let color_hair = rgb(0, 0, 0); // TODO: Player.HairColor
    let color_headblood = rgb(172, 169, 168);
    let alpha_nades: u8;

    if soldier.has_cigar == 5 {
        color_cygar = rgb(97, 97, 97);
    }

    let realistic_mode = false; // TODO: use real value

    if soldier.health > (90.0 - 40.0 * f32::from(realistic_mode as u8)) {
        alpha_blood = 0;
    }

    if realistic_mode && soldier.visible > 0 && soldier.visible < 45 && soldier.alpha > 60 {
        // TODO: if this really needs to change it should be done somewhere during update, not here
        // soldier.alpha = 3 * soldier.visible;
        alpha_base = 3 * soldier.visible;
        alpha_blood = 0;
    }

    alpha_nades = (0.75 * f32::from(alpha_base)).round() as u8;

    (
        [
            color_none,
            color_main,
            color_pants,
            color_skin,
            color_hair,
            color_cygar,
            color_headblood,
        ],
        [alpha_base, alpha_blood, alpha_nades],
    )
}

fn parts_visibility(base_visibility: &BitSet, soldier: &Soldier, blood: bool) -> BitSet {
    let mut visible = base_visibility.clone();

    if blood {
        visible.set(SoldierPart::LeftThighDmg.id(), true);
        visible.set(SoldierPart::LeftLowerlegDmg.id(), true);
        visible.set(SoldierPart::LeftForearmDmg.id(), true);
        visible.set(SoldierPart::LeftArmDmg.id(), true);
        visible.set(SoldierPart::ChestDmg.id(), true);
        visible.set(SoldierPart::HipDmg.id(), true);
        visible.set(SoldierPart::HeadDmg.id(), true);
        visible.set(SoldierPart::RightThighDmg.id(), true);
        visible.set(SoldierPart::RightLowerlegDmg.id(), true);
        visible.set(SoldierPart::RightForearmDmg.id(), true);
        visible.set(SoldierPart::RightArmDmg.id(), true);
    }

    if soldier.control.jets && soldier.jets_count > 0 {
        visible.set(SoldierPart::LeftFoot.id(), false);
        visible.set(SoldierPart::RightFoot.id(), false);
        visible.set(SoldierPart::LeftJetfoot.id(), true);
        visible.set(SoldierPart::RightJetfoot.id(), true);
    }

    if soldier.vest > 0.0 {
        visible.set(SoldierPart::Vest.id(), true);
    }

    let index = if soldier.tertiary_weapon().kind == WeaponKind::FragGrenade {
        SoldierPart::FragGrenade1.id()
    } else {
        SoldierPart::ClusterGrenade1.id()
    };

    let ammo = soldier.tertiary_weapon().ammo_count as i32;

    let n = if soldier.body_animation.id == Anim::Throw {
        i32::min(5, ammo - 1)
    } else {
        i32::min(5, ammo)
    };

    for i in 0..n {
        visible.set(index + i as usize, true);
    }

    let chain = 0; // TODO: Player.Chain (this seems broken, check skeleton)

    match chain {
        1 => {
            visible.set(SoldierPart::SilverLchain.id(), true);
            visible.set(SoldierPart::SilverRchain.id(), true);
            visible.set(SoldierPart::SilverPendant.id(), true);
        }
        2 => {
            visible.set(SoldierPart::GoldenLchain.id(), true);
            visible.set(SoldierPart::GoldenRchain.id(), true);
            visible.set(SoldierPart::GoldenPendant.id(), true);
        }
        _ => {}
    }

    if soldier.has_cigar == 5 || soldier.has_cigar == 10 {
        visible.set(SoldierPart::Cigar.id(), true);
    }

    if soldier.dead_meat {
        visible.set(SoldierPart::Head.id(), false);
        visible.set(SoldierPart::HeadDmg.id(), false);
        visible.set(SoldierPart::HeadDead.id(), true);
        visible.set(SoldierPart::HeadDeadDmg.id(), true);
    }

    if soldier.primary_weapon().kind == WeaponKind::Bow
        || soldier.primary_weapon().kind == WeaponKind::FlameBow
    {
        visible.set(SoldierPart::RamboBadge.id(), true);
    } else {
        let grabbed = match soldier.body_animation.id {
            Anim::Wipe | Anim::TakeOff => soldier.body_animation.frame > 4,
            _ => false,
        };

        if soldier.wear_helmet == 1 {
            let head_cap = gfx::Soldier::Helm; // TODO: Player.HeadCap

            match head_cap {
                gfx::Soldier::Helm if grabbed => visible.set(SoldierPart::GrabbedHelmet.id(), true),
                gfx::Soldier::Kap if grabbed => visible.set(SoldierPart::GrabbedHat.id(), true),
                gfx::Soldier::Helm if !grabbed => visible.set(SoldierPart::Helmet.id(), true),
                gfx::Soldier::Kap if !grabbed => visible.set(SoldierPart::Hat.id(), true),
                _ => {}
            }
        }

        let hair_style = 3; // TODO: Player.HairStyle

        if grabbed || soldier.wear_helmet != 1 || hair_style == 3 {
            match hair_style {
                1 => for i in 0..6 {
                    visible.set(SoldierPart::HairDreadlocks.id() + i, true);
                },
                2 => visible.set(SoldierPart::HairPunk.id(), true),
                3 => visible.set(SoldierPart::MrT.id(), true),
                4 => visible.set(SoldierPart::HairNormal.id(), true),
                _ => {}
            }
        }
    }

    // secondary weapon (on the back)

    let index = soldier.secondary_weapon().kind.index();

    if index >= WeaponKind::DesertEagles.index() && index <= WeaponKind::Flamer.index() {
        visible.set(SoldierPart::SecondaryDeagles.id() + index, true);
    }

    // primary weapon

    let weapon = soldier.primary_weapon();
    let ammo = weapon.ammo_count;
    let reload_count = weapon.reload_time_count;

    if weapon.kind == WeaponKind::Minigun {
        visible.set(SoldierPart::PrimaryMinigun.id(), true);

        if ammo > 0 || (ammo == 0 && weapon.reload_time_count < 65) {
            visible.set(SoldierPart::PrimaryMinigunClip.id(), true);
        }

        if soldier.fired > 0 {
            visible.set(SoldierPart::PrimaryMinigunFire.id(), true);
        }
    } else if weapon.kind == WeaponKind::Bow || weapon.kind == WeaponKind::FlameBow {
        if ammo == 0 {
            visible.set(SoldierPart::PrimaryBowArrowReload.id(), true);
        } else {
            visible.set(SoldierPart::PrimaryBowArrow.id(), true);
        }

        if soldier.body_animation.id == Anim::ReloadBow {
            visible.set(SoldierPart::PrimaryBowReload.id(), true);
            visible.set(SoldierPart::PrimaryBowStringReload.id(), true);
        } else {
            visible.set(SoldierPart::PrimaryBow.id(), true);
            visible.set(SoldierPart::PrimaryBowString.id(), true);
        }

        if soldier.fired > 0 {
            visible.set(SoldierPart::PrimaryBowFire.id(), true);
        }
    } else if !soldier.dead_meat {
        let first = SoldierPart::PrimaryDeagles;
        let mut index = weapon.kind.index();

        if index >= WeaponKind::DesertEagles.index() && index <= WeaponKind::Flamer.index() {
            if weapon.kind == WeaponKind::Flamer {
                index = SoldierPart::PrimaryFlamer.id() - first.id();
            } else {
                index *= 3;
            }

            visible.set(first.id() + index, true);

            if weapon.clip_sprite.is_some()
                && (ammo > 0
                    || ammo == 0
                        && (reload_count < weapon.clip_in_time
                            || reload_count > weapon.clip_out_time))
            {
                visible.set(first.id() + index + 1, true);
            }

            if soldier.fired > 0 {
                visible.set(first.id() + 2, true);
            }
        }
    }

    visible
}

pub fn render_skeleton(soldier: &Soldier, batch: &mut DrawBatch, px: f32, frame_percent: f32) {
    let sk = &soldier.skeleton;

    for constraint in sk.constraints() {
        let pa = constraint.particle_num.0 as usize;
        let pb = constraint.particle_num.1 as usize;

        let a = lerp(sk.old_pos(pa), sk.pos(pa), frame_percent);
        let b = lerp(sk.old_pos(pb), sk.pos(pb), frame_percent);

        let m = Transform::WithPivot {
            pos: a,
            pivot: vec2(0.0, 0.0),
            scale: vec2(distance(a, b), 1.0),
            rot: vec2angle(b - a),
        }.matrix();

        batch.add_quad(
            None,
            &[
                vertex(m * vec2(0.0, -0.5 * px), Vec2::zero(), rgb(255, 255, 0)),
                vertex(m * vec2(1.0, -0.5 * px), Vec2::zero(), rgb(255, 255, 0)),
                vertex(m * vec2(1.0, 0.5 * px), Vec2::zero(), rgb(255, 255, 0)),
                vertex(m * vec2(0.0, 0.5 * px), Vec2::zero(), rgb(255, 255, 0)),
            ],
        );
    }

    for particle in sk.particles() {
        let p = lerp(particle.old_pos, particle.pos, frame_percent);
        let m = Mat2d::translate(p.x, p.y);

        batch.add_quad(
            None,
            &[
                vertex(m * vec2(-1.0 * px, -1.0 * px), Vec2::zero(), rgb(0, 0, 255)),
                vertex(m * vec2(1.0 * px, -1.0 * px), Vec2::zero(), rgb(0, 0, 255)),
                vertex(m * vec2(1.0 * px, 1.0 * px), Vec2::zero(), rgb(0, 0, 255)),
                vertex(m * vec2(-1.0 * px, 1.0 * px), Vec2::zero(), rgb(0, 0, 255)),
            ],
        );
    }
}
