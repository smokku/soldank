use super::*;
use bit_array::BitArray;
use gfx::{GostekPart, SpriteData};
use ini::Ini;
use shared::soldier::Soldier;
use std::str::FromStr;
use typenum::U256;

type BitSet = BitArray<u64, U256>;

#[derive(Debug, Copy, Clone)]
pub enum GostekSprite {
    None,
    Gostek(gfx::Gostek),
    Weapon(gfx::Weapon),
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
            base_visibility: BitSet::from_fn(|i| {
                GostekPart::data().get(i).map_or(false, |p| p.visible)
            }),
        }
    }

    pub fn load_data(&mut self, cfg: &Ini) {
        self.data = GostekPart::data().to_vec();

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

    pub fn render(
        &self,
        soldier: &Soldier,
        batch: &mut DrawBatch,
        sprites: &[Vec<Sprite>],
        frame_percent: f32,
    ) {
        let sk = &soldier.skeleton;
        let (colors, alpha) = Self::colors_and_alpha(soldier);
        let visible = self.parts_visibility(soldier, alpha[GostekAlpha::Blood as usize] > 0);

        // TODO: team2 offset, dredlock rotation matrix

        for (i, part) in self.data.iter().enumerate() {
            if visible[i] && !part.sprite.is_none() {
                let mut sprite_index: usize = 0;
                let mut cx = part.center.0;
                let mut cy = part.center.1;
                let mut scale = vec2(1.0, 1.0);
                let p0 = lerp(
                    sk.old_pos[part.point.0],
                    sk.pos[part.point.0],
                    frame_percent,
                );
                let p1 = lerp(
                    sk.old_pos[part.point.1],
                    sk.pos[part.point.1],
                    frame_percent,
                );
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
                    scale.x = f32::min(1.5, (p1 - p0).magnitude() / part.flexibility);
                }

                let color = {
                    let color = colors[part.color as usize];
                    rgba(color.r(), color.g(), color.b(), alpha[part.alpha as usize])
                };

                let sprite = match part.sprite {
                    GostekSprite::Gostek(part_sprite) => {
                        let group = gfx::Group::Gostek;
                        let sprite = part_sprite + sprite_index;
                        &sprites[group.id()][sprite.id()]
                    }
                    GostekSprite::Weapon(part_sprite) => {
                        let group = gfx::Group::Weapon;
                        let sprite = part_sprite + sprite_index;
                        &sprites[group.id()][sprite.id()]
                    }
                    GostekSprite::None => unreachable!(),
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

    fn parts_visibility(&self, soldier: &Soldier, blood: bool) -> BitSet {
        let mut visible = self.base_visibility.clone();

        if blood {
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

        let index = if soldier.tertiary_weapon().kind == WeaponKind::FragGrenade {
            GostekPart::FragGrenade1.id()
        } else {
            GostekPart::ClusterGrenade1.id()
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
                visible.set(GostekPart::SilverLchain.id(), true);
                visible.set(GostekPart::SilverRchain.id(), true);
                visible.set(GostekPart::SilverPendant.id(), true);
            }
            2 => {
                visible.set(GostekPart::GoldenLchain.id(), true);
                visible.set(GostekPart::GoldenRchain.id(), true);
                visible.set(GostekPart::GoldenPendant.id(), true);
            }
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

        if soldier.primary_weapon().kind == WeaponKind::Bow
            || soldier.primary_weapon().kind == WeaponKind::FlameBow
        {
            visible.set(GostekPart::RamboBadge.id(), true);
        } else {
            let grabbed = match soldier.body_animation.id {
                Anim::Wipe | Anim::TakeOff => soldier.body_animation.frame > 4,
                _ => false,
            };

            if soldier.wear_helmet == 1 {
                let head_cap = gfx::Gostek::Helm; // TODO: Player.HeadCap

                match head_cap {
                    gfx::Gostek::Helm if grabbed => {
                        visible.set(GostekPart::GrabbedHelmet.id(), true)
                    }
                    gfx::Gostek::Kap if grabbed => visible.set(GostekPart::GrabbedHat.id(), true),
                    gfx::Gostek::Helm if !grabbed => visible.set(GostekPart::Helmet.id(), true),
                    gfx::Gostek::Kap if !grabbed => visible.set(GostekPart::Hat.id(), true),
                    _ => {}
                }
            }

            let hair_style = 3; // TODO: Player.HairStyle

            if grabbed || soldier.wear_helmet != 1 || hair_style == 3 {
                match hair_style {
                    1 => for i in 0..6 {
                        visible.set(GostekPart::HairDreadlocks.id() + i, true);
                    },
                    2 => visible.set(GostekPart::HairPunk.id(), true),
                    3 => visible.set(GostekPart::MrT.id(), true),
                    4 => visible.set(GostekPart::HairNormal.id(), true),
                    _ => {}
                }
            }
        }

        // secondary weapon (on the back)

        let index = soldier.secondary_weapon().kind.index();

        if index >= WeaponKind::DesertEagles.index() && index <= WeaponKind::Flamer.index() {
            visible.set(GostekPart::SecondaryDeagles.id() + index, true);
        }

        // primary weapon

        let weapon = soldier.primary_weapon();
        let ammo = weapon.ammo_count;
        let reload_count = weapon.reload_time_count;

        if weapon.kind == WeaponKind::Minigun {
            visible.set(GostekPart::PrimaryMinigun.id(), true);

            if ammo > 0 || (ammo == 0 && weapon.reload_time_count < 65) {
                visible.set(GostekPart::PrimaryMinigunClip.id(), true);
            }

            if soldier.fired > 0 {
                visible.set(GostekPart::PrimaryMinigunFire.id(), true);
            }
        } else if weapon.kind == WeaponKind::Bow || weapon.kind == WeaponKind::FlameBow {
            if ammo == 0 {
                visible.set(GostekPart::PrimaryBowArrowReload.id(), true);
            } else {
                visible.set(GostekPart::PrimaryBowArrow.id(), true);
            }

            if soldier.body_animation.id == Anim::ReloadBow {
                visible.set(GostekPart::PrimaryBowReload.id(), true);
                visible.set(GostekPart::PrimaryBowStringReload.id(), true);
            } else {
                visible.set(GostekPart::PrimaryBow.id(), true);
                visible.set(GostekPart::PrimaryBowString.id(), true);
            }

            if soldier.fired > 0 {
                visible.set(GostekPart::PrimaryBowFire.id(), true);
            }
        } else if !soldier.dead_meat {
            let first = GostekPart::PrimaryDeagles;
            let mut index = weapon.kind.index();

            if index >= WeaponKind::DesertEagles.index() && index <= WeaponKind::Flamer.index() {
                if weapon.kind == WeaponKind::Flamer {
                    index = GostekPart::PrimaryFlamer.id() - first.id();
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
        let n = sk.constraint_count as usize;

        for constraint in &soldier.skeleton.constraints[1..n + 1] {
            let pa = constraint.part_a as usize;
            let pb = constraint.part_b as usize;

            let a = lerp(sk.old_pos[pa], sk.pos[pa], frame_percent);
            let b = lerp(sk.old_pos[pb], sk.pos[pb], frame_percent);

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

        for (a, b) in sk.old_pos[1..25].iter().zip(&sk.pos[1..25]) {
            let p = lerp(*a, *b, frame_percent);
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
}
