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
        for (i, part) in self.data.iter().enumerate() {
            if self.base_visibility.get(i).unwrap() && !part.sprite.is_none() {
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

                match part.sprite {
                    GostekSprite::Gostek(spriteid) => {
                        let spriteid = spriteid + sid;
                        let sprite = &sprites[spriteid.group().id()][spriteid.id()];
                        let (w, h) = (sprite.width, sprite.height);

                        batch.add_tinted_sprite(sprite, rgb(255, 255, 255), Transform::WithPivot {
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
