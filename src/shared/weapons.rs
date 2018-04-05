use super::*;

const SECOND: u16 = 60;
const BULLET_TIMEOUT: u16 = SECOND * 7;
const GRENADE_TIMEOUT: u16 = SECOND * 3;
const M2BULLET_TIMEOUT: u16 = SECOND;
const FLAMER_TIMEOUT: u16 = SECOND * 32;
const MELEE_TIMEOUT: u16 = 1;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WeaponGroup {
    Primary,
    Secondary,
    Other,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WeaponKind {
    DesertEagles,
    MP5,
    Ak74,
    SteyrAUG,
    Spas12,
    Ruger77,
    M79,
    Barrett,
    Minimi,
    Minigun,
    USSOCOM,
    Knife,
    Chainsaw,
    LAW,
    FlameBow,
    Bow,
    Flamer,
    M2,
    NoWeapon,
    FragGrenade,
    ClusterGrenade,
    Cluster,
    ThrownKnife,
}

impl Default for WeaponKind {
    fn default() -> WeaponKind {
        WeaponKind::NoWeapon
    }
}

impl WeaponKind {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn values() -> &'static [WeaponKind] {
        static VALUES: &[WeaponKind] = &[
            WeaponKind::DesertEagles,
            WeaponKind::MP5,
            WeaponKind::Ak74,
            WeaponKind::SteyrAUG,
            WeaponKind::Spas12,
            WeaponKind::Ruger77,
            WeaponKind::M79,
            WeaponKind::Barrett,
            WeaponKind::Minimi,
            WeaponKind::Minigun,
            WeaponKind::USSOCOM,
            WeaponKind::Knife,
            WeaponKind::Chainsaw,
            WeaponKind::LAW,
            WeaponKind::FlameBow,
            WeaponKind::Bow,
            WeaponKind::Flamer,
            WeaponKind::M2,
            WeaponKind::NoWeapon,
            WeaponKind::FragGrenade,
            WeaponKind::ClusterGrenade,
            WeaponKind::Cluster,
            WeaponKind::ThrownKnife,
        ];

        VALUES
    }

    #[allow(dead_code)]
    pub fn group(&self) -> WeaponGroup {
        match *self {
            WeaponKind::DesertEagles => WeaponGroup::Primary,
            WeaponKind::MP5 => WeaponGroup::Primary,
            WeaponKind::Ak74 => WeaponGroup::Primary,
            WeaponKind::SteyrAUG => WeaponGroup::Primary,
            WeaponKind::Spas12 => WeaponGroup::Primary,
            WeaponKind::Ruger77 => WeaponGroup::Primary,
            WeaponKind::M79 => WeaponGroup::Primary,
            WeaponKind::Barrett => WeaponGroup::Primary,
            WeaponKind::Minimi => WeaponGroup::Primary,
            WeaponKind::Minigun => WeaponGroup::Primary,
            WeaponKind::USSOCOM => WeaponGroup::Secondary,
            WeaponKind::Knife => WeaponGroup::Secondary,
            WeaponKind::Chainsaw => WeaponGroup::Secondary,
            WeaponKind::LAW => WeaponGroup::Secondary,
            WeaponKind::FlameBow => WeaponGroup::Other,
            WeaponKind::Bow => WeaponGroup::Other,
            WeaponKind::Flamer => WeaponGroup::Other,
            WeaponKind::M2 => WeaponGroup::Other,
            WeaponKind::NoWeapon => WeaponGroup::Other,
            WeaponKind::FragGrenade => WeaponGroup::Other,
            WeaponKind::ClusterGrenade => WeaponGroup::Other,
            WeaponKind::Cluster => WeaponGroup::Other,
            WeaponKind::ThrownKnife => WeaponGroup::Other,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Weapon {
    pub kind: WeaponKind,
    pub ammo: u8,
    pub ammo_count: u8,
    pub movement_acc: f32,
    pub bink: i16,
    pub recoil: u16,
    pub fire_interval: u16,
    pub fire_interval_prev: u16,
    pub fire_interval_count: u16,
    pub fire_interval_real: f32,
    pub start_up_time: u16,
    pub start_up_time_count: u16,
    pub reload_time: u16,
    pub reload_time_prev: u16,
    pub reload_time_count: u16,
    pub reload_time_real: f32,
    pub clip_reload: bool,
    pub clip_in_time: u16,
    pub clip_out_time: u16,
    pub name: &'static str,
    pub ini_name: &'static str,
    pub speed: f32,
    pub hit_multiply: f32,
    pub bullet_spread: f32,
    pub push: f32,
    pub inherited_velocity: f32,
    pub modifier_legs: f32,
    pub modifier_chest: f32,
    pub modifier_head: f32,
    pub fire_mode: u8,
    pub timeout: u16,
    pub bullet_style: u8,
    pub sprite: Option<gfx::Weapon>,
    pub clip_sprite: Option<gfx::Weapon>,
    pub fire_sprite: Option<gfx::Weapon>,
    pub bullet_sprite: Option<gfx::Weapon>,
}

impl Weapon {
    pub fn new(kind: WeaponKind, realistic: bool) -> Weapon {
        let mut weapon = Weapon {
            kind,
            ..Default::default()
        };

        match kind {
            WeaponKind::DesertEagles => {
                weapon.name = "Desert Eagles";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 2;
                weapon.sprite = Some(gfx::Weapon::Deagles);
                weapon.clip_sprite = Some(gfx::Weapon::DeaglesClip);
                weapon.bullet_sprite = Some(gfx::Weapon::DeaglesBullet);
                weapon.fire_sprite = Some(gfx::Weapon::DeaglesFire);

                if realistic {
                    weapon.hit_multiply = 1.66;
                    weapon.fire_interval = 27;
                    weapon.ammo = 7;
                    weapon.reload_time = 106;
                    weapon.speed = 19.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.02;
                    weapon.bullet_spread = 0.1;
                    weapon.recoil = 55;
                    weapon.push = 0.0164;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1.81;
                    weapon.fire_interval = 24;
                    weapon.ammo = 7;
                    weapon.reload_time = 87;
                    weapon.speed = 19.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.009;
                    weapon.bullet_spread = 0.15;
                    weapon.recoil = 0;
                    weapon.push = 0.0176;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::MP5 => {
                weapon.name = "HK MP5";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Mp5);
                weapon.clip_sprite = Some(gfx::Weapon::Mp5Clip);
                weapon.bullet_sprite = Some(gfx::Weapon::Mp5Bullet);
                weapon.fire_sprite = Some(gfx::Weapon::Mp5Fire);

                if realistic {
                    weapon.hit_multiply = 0.94;
                    weapon.fire_interval = 6;
                    weapon.ammo = 30;
                    weapon.reload_time = 110;
                    weapon.speed = 18.9;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = -10;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.03;
                    weapon.recoil = 9;
                    weapon.push = 0.0164;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1.01;
                    weapon.fire_interval = 6;
                    weapon.ammo = 30;
                    weapon.reload_time = 105;
                    weapon.speed = 18.9;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.14;
                    weapon.recoil = 0;
                    weapon.push = 0.0112;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::Ak74 => {
                weapon.name = "Ak-74";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Ak74);
                weapon.clip_sprite = Some(gfx::Weapon::Ak74Clip);
                weapon.bullet_sprite = Some(gfx::Weapon::Ak74Bullet);
                weapon.fire_sprite = Some(gfx::Weapon::Ak74Fire);

                if realistic {
                    weapon.hit_multiply = 1.08;
                    weapon.fire_interval = 11;
                    weapon.ammo = 35;
                    weapon.reload_time = 158;
                    weapon.speed = 24.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = -10;
                    weapon.movement_acc = 0.02;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 13;
                    weapon.push = 0.0132;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1.004;
                    weapon.fire_interval = 10;
                    weapon.ammo = 35;
                    weapon.reload_time = 165;
                    weapon.speed = 24.6;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = -12;
                    weapon.movement_acc = 0.011;
                    weapon.bullet_spread = 0.025;
                    weapon.recoil = 0;
                    weapon.push = 0.01376;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::SteyrAUG => {
                weapon.name = "Steyr AUG";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Steyr);
                weapon.clip_sprite = Some(gfx::Weapon::SteyrClip);
                weapon.bullet_sprite = Some(gfx::Weapon::SteyrBullet);
                weapon.fire_sprite = Some(gfx::Weapon::SteyrFire);

                if realistic {
                    weapon.hit_multiply = 0.68;
                    weapon.fire_interval = 7;
                    weapon.ammo = 30;
                    weapon.reload_time = 126;
                    weapon.speed = 26.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = -9;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 11;
                    weapon.push = 0.012;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 0.71;
                    weapon.fire_interval = 7;
                    weapon.ammo = 25;
                    weapon.reload_time = 125;
                    weapon.speed = 26.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.075;
                    weapon.recoil = 0;
                    weapon.push = 0.0084;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::Spas12 => {
                weapon.name = "Spas-12";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = false;
                weapon.fire_mode = 2;
                weapon.sprite = Some(gfx::Weapon::Spas);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::SpasFire);

                if realistic {
                    weapon.hit_multiply = 1.2;
                    weapon.fire_interval = 35;
                    weapon.ammo = 7;
                    weapon.reload_time = 175;
                    weapon.speed = 13.2;
                    weapon.bullet_style = 3;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.8;
                    weapon.recoil = 65;
                    weapon.push = 0.0224;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1.22;
                    weapon.fire_interval = 32;
                    weapon.ammo = 7;
                    weapon.reload_time = 175;
                    weapon.speed = 14.0;
                    weapon.bullet_style = 3;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.8;
                    weapon.recoil = 0;
                    weapon.push = 0.0188;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::Ruger77 => {
                weapon.name = "Ruger 77";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = false;
                weapon.fire_mode = 2;
                weapon.sprite = Some(gfx::Weapon::Ruger);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = Some(gfx::Weapon::RugerBullet);
                weapon.fire_sprite = Some(gfx::Weapon::RugerFire);

                if realistic {
                    weapon.hit_multiply = 2.22;
                    weapon.fire_interval = 52;
                    weapon.ammo = 4;
                    weapon.reload_time = 104;
                    weapon.speed = 33.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 14;
                    weapon.movement_acc = 0.03;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 54;
                    weapon.push = 0.0096;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 2.49;
                    weapon.fire_interval = 45;
                    weapon.ammo = 4;
                    weapon.reload_time = 78;
                    weapon.speed = 33.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.03;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.012;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.2;
                    weapon.modifier_chest = 1.05;
                    weapon.modifier_legs = 1.0;
                }
            }

            WeaponKind::M79 => {
                weapon.name = "M79";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::M79);
                weapon.clip_sprite = Some(gfx::Weapon::M79Clip);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::M79Fire);

                if realistic {
                    weapon.hit_multiply = 1600.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 173;
                    weapon.speed = 11.4;
                    weapon.bullet_style = 4;
                    weapon.start_up_time = 0;
                    weapon.bink = 45;
                    weapon.movement_acc = 0.03;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 420;
                    weapon.push = 0.024;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1550.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 178;
                    weapon.speed = 10.7;
                    weapon.bullet_style = 4;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.036;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::Barrett => {
                weapon.name = "Barrett M82A1";
                weapon.ini_name = "Barret M82A1";
                weapon.clip_reload = true;
                weapon.fire_mode = 2;
                weapon.sprite = Some(gfx::Weapon::Barrett);
                weapon.clip_sprite = Some(gfx::Weapon::BarrettClip);
                weapon.bullet_sprite = Some(gfx::Weapon::BarrettBullet);
                weapon.fire_sprite = Some(gfx::Weapon::BarrettFire);

                if realistic {
                    weapon.hit_multiply = 4.95;
                    weapon.fire_interval = 200;
                    weapon.ammo = 10;
                    weapon.reload_time = 170;
                    weapon.speed = 55.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 16;
                    weapon.bink = 80;
                    weapon.movement_acc = 0.07;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0056;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 4.45;
                    weapon.fire_interval = 225;
                    weapon.ammo = 10;
                    weapon.reload_time = 70;
                    weapon.speed = 55.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 19;
                    weapon.bink = 65;
                    weapon.movement_acc = 0.05;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.018;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.0;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 1.0;
                }
            }

            WeaponKind::Minimi => {
                weapon.name = "FN Minimi";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Minimi);
                weapon.clip_sprite = Some(gfx::Weapon::MinimiClip);
                weapon.bullet_sprite = Some(gfx::Weapon::MinimiBullet);
                weapon.fire_sprite = Some(gfx::Weapon::MinimiFire);

                if realistic {
                    weapon.hit_multiply = 0.81;
                    weapon.fire_interval = 10;
                    weapon.ammo = 50;
                    weapon.reload_time = 261;
                    weapon.speed = 27.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = -8;
                    weapon.movement_acc = 0.02;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 8;
                    weapon.push = 0.0116;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 0.85;
                    weapon.fire_interval = 9;
                    weapon.ammo = 50;
                    weapon.reload_time = 250;
                    weapon.speed = 27.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.013;
                    weapon.bullet_spread = 0.064;
                    weapon.recoil = 0;
                    weapon.push = 0.0128;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::Minigun => {
                weapon.name = "XM214 Minigun";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Minigun);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = Some(gfx::Weapon::MinigunBullet);
                weapon.fire_sprite = Some(gfx::Weapon::MinigunFire);

                if realistic {
                    weapon.hit_multiply = 0.43;
                    weapon.fire_interval = 4;
                    weapon.ammo = 100;
                    weapon.reload_time = 320;
                    weapon.speed = 29.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 33;
                    weapon.bink = -2;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.1;
                    weapon.recoil = 4;
                    weapon.push = 0.0108;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 0.468;
                    weapon.fire_interval = 3;
                    weapon.ammo = 100;
                    weapon.reload_time = 480;
                    weapon.speed = 29.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 25;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0625;
                    weapon.bullet_spread = 0.3;
                    weapon.recoil = 0;
                    weapon.push = 0.0104;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::USSOCOM => {
                weapon.name = "USSOCOM";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = true;
                weapon.fire_mode = 2;
                weapon.sprite = Some(gfx::Weapon::Socom);
                weapon.clip_sprite = Some(gfx::Weapon::SocomClip);
                weapon.bullet_sprite = Some(gfx::Weapon::ColtBullet);
                weapon.fire_sprite = Some(gfx::Weapon::SocomFire);

                if realistic {
                    weapon.hit_multiply = 1.30;
                    weapon.fire_interval = 12;
                    weapon.ammo = 12;
                    weapon.reload_time = 72;
                    weapon.speed = 18.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.02;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 28;
                    weapon.push = 0.0172;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1.49;
                    weapon.fire_interval = 10;
                    weapon.ammo = 14;
                    weapon.reload_time = 60;
                    weapon.speed = 18.0;
                    weapon.bullet_style = 1;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.02;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::Knife => {
                weapon.name = "Combat Knife";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Knife);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = None;

                if realistic {
                    weapon.hit_multiply = 2250.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 3;
                    weapon.speed = 6.0;
                    weapon.bullet_style = 11;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.028;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 2150.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 3;
                    weapon.speed = 6.0;
                    weapon.bullet_style = 11;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.12;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::Chainsaw => {
                weapon.name = "Chainsaw";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Chainsaw);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::ChainsawFire);

                if realistic {
                    weapon.hit_multiply = 21.0;
                    weapon.fire_interval = 2;
                    weapon.ammo = 200;
                    weapon.reload_time = 110;
                    weapon.speed = 7.6;
                    weapon.bullet_style = 11;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 1;
                    weapon.push = 0.0028;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 50.0;
                    weapon.fire_interval = 2;
                    weapon.ammo = 200;
                    weapon.reload_time = 110;
                    weapon.speed = 8.0;
                    weapon.bullet_style = 11;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0028;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::LAW => {
                weapon.name = "LAW";
                weapon.ini_name = "M72 LAW";
                weapon.clip_reload = true;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Law);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::LawFire);

                if realistic {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 30;
                    weapon.ammo = 1;
                    weapon.reload_time = 495;
                    weapon.speed = 23.0;
                    weapon.bullet_style = 12;
                    weapon.start_up_time = 12;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 9;
                    weapon.push = 0.012;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1550.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 300;
                    weapon.speed = 23.0;
                    weapon.bullet_style = 12;
                    weapon.start_up_time = 13;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.028;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::FlameBow => {
                weapon.name = "Flame Bow";
                weapon.ini_name = "Flamed Arrows";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Bow);
                weapon.clip_sprite = Some(gfx::Weapon::BowS);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::BowFire);

                if realistic {
                    weapon.hit_multiply = 8.0;
                    weapon.fire_interval = 10;
                    weapon.ammo = 1;
                    weapon.reload_time = 39;
                    weapon.speed = 18.0;
                    weapon.bullet_style = 8;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 8.0;
                    weapon.fire_interval = 10;
                    weapon.ammo = 1;
                    weapon.reload_time = 39;
                    weapon.speed = 18.0;
                    weapon.bullet_style = 8;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::Bow => {
                weapon.name = "Bow";
                weapon.ini_name = "Rambo Bow";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Bow);
                weapon.clip_sprite = Some(gfx::Weapon::BowS);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::BowFire);

                if realistic {
                    weapon.hit_multiply = 12.0;
                    weapon.fire_interval = 10;
                    weapon.ammo = 1;
                    weapon.reload_time = 25;
                    weapon.speed = 21.0;
                    weapon.bullet_style = 7;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0148;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 12.0;
                    weapon.fire_interval = 10;
                    weapon.ammo = 1;
                    weapon.reload_time = 25;
                    weapon.speed = 21.0;
                    weapon.bullet_style = 7;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0148;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::Flamer => {
                weapon.name = "Flamer";
                weapon.ini_name = weapon.name;
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Flamer);
                weapon.clip_sprite = Some(gfx::Weapon::Flamer);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::FlamerFire);

                if realistic {
                    weapon.hit_multiply = 12.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 200;
                    weapon.reload_time = 5;
                    weapon.speed = 12.5;
                    weapon.bullet_style = 5;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.016;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 19.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 200;
                    weapon.reload_time = 5;
                    weapon.speed = 10.5;
                    weapon.bullet_style = 5;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.016;
                    weapon.inherited_velocity = 0.5;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::M2 => {
                weapon.name = "M2 MG";
                weapon.ini_name = "Stationary Gun";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Minigun);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = None;

                if realistic {
                    weapon.hit_multiply = 1.55;
                    weapon.fire_interval = 14;
                    weapon.ammo = 100;
                    weapon.reload_time = 366;
                    weapon.speed = 36.0;
                    weapon.bullet_style = 14;
                    weapon.start_up_time = 21;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0088;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1.8;
                    weapon.fire_interval = 10;
                    weapon.ammo = 100;
                    weapon.reload_time = 366;
                    weapon.speed = 36.0;
                    weapon.bullet_style = 14;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0088;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 0.95;
                    weapon.modifier_legs = 0.85;
                }
            }

            WeaponKind::NoWeapon => {
                weapon.name = "Hands";
                weapon.ini_name = "Punch";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = None;
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = None;

                if realistic {
                    weapon.hit_multiply = 330.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 3;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 6;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 330.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 3;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 6;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }

            WeaponKind::FragGrenade => {
                weapon.name = "Frag Grenade";
                weapon.ini_name = "Grenade";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::FragGrenade);
                weapon.clip_sprite = Some(gfx::Weapon::FragGrenade);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::Ak74Fire);

                if realistic {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 80;
                    weapon.ammo = 1;
                    weapon.reload_time = 20;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 2;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 1.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 80;
                    weapon.ammo = 1;
                    weapon.reload_time = 20;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 2;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 1.0;
                    weapon.modifier_head = 1.0;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 1.0;
                }
            }

            WeaponKind::ClusterGrenade => {
                weapon.name = "Frag Grenade";
                weapon.ini_name = "";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::FragGrenade);
                weapon.clip_sprite = Some(gfx::Weapon::FragGrenade);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::Ak74Fire);

                if realistic {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 80;
                    weapon.ammo = 1;
                    weapon.reload_time = 20;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 9;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 1.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 80;
                    weapon.ammo = 1;
                    weapon.reload_time = 20;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 9;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 1.0;
                    weapon.modifier_head = 1.0;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 1.0;
                }
            }

            WeaponKind::Cluster => {
                weapon.name = "Frag Grenade";
                weapon.ini_name = "";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::FragGrenade);
                weapon.clip_sprite = Some(gfx::Weapon::FragGrenade);
                weapon.bullet_sprite = None;
                weapon.fire_sprite = Some(gfx::Weapon::Ak74Fire);

                if realistic {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 80;
                    weapon.ammo = 1;
                    weapon.reload_time = 20;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 10;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 1.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 1500.0;
                    weapon.fire_interval = 80;
                    weapon.ammo = 1;
                    weapon.reload_time = 20;
                    weapon.speed = 5.0;
                    weapon.bullet_style = 10;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.0;
                    weapon.inherited_velocity = 1.0;
                    weapon.modifier_head = 1.0;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 1.0;
                }
            }

            WeaponKind::ThrownKnife => {
                weapon.name = "Combat Knife";
                weapon.ini_name = "";
                weapon.clip_reload = false;
                weapon.fire_mode = 0;
                weapon.sprite = Some(gfx::Weapon::Knife);
                weapon.clip_sprite = None;
                weapon.bullet_sprite = None;
                weapon.fire_sprite = None;

                if realistic {
                    weapon.hit_multiply = 2250.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 3;
                    weapon.speed = 6.0;
                    weapon.bullet_style = 13;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.01;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 10;
                    weapon.push = 0.028;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.1;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.6;
                } else {
                    weapon.hit_multiply = 2150.0;
                    weapon.fire_interval = 6;
                    weapon.ammo = 1;
                    weapon.reload_time = 3;
                    weapon.speed = 6.0;
                    weapon.bullet_style = 13;
                    weapon.start_up_time = 0;
                    weapon.bink = 0;
                    weapon.movement_acc = 0.0;
                    weapon.bullet_spread = 0.0;
                    weapon.recoil = 0;
                    weapon.push = 0.12;
                    weapon.inherited_velocity = 0.0;
                    weapon.modifier_head = 1.15;
                    weapon.modifier_chest = 1.0;
                    weapon.modifier_legs = 0.9;
                }
            }
        };

        weapon.fire_interval_prev = weapon.fire_interval;
        weapon.fire_interval_count = weapon.fire_interval;
        weapon.fire_interval_real = f32::from(weapon.fire_interval);
        weapon.ammo_count = weapon.ammo;
        weapon.reload_time_prev = weapon.reload_time;
        weapon.reload_time_count = weapon.reload_time;
        weapon.reload_time_real = f32::from(weapon.reload_time);
        weapon.start_up_time_count = weapon.start_up_time;

        if weapon.clip_reload {
            weapon.clip_out_time = (f32::from(weapon.reload_time) * 0.8).trunc() as u16;
            weapon.clip_in_time = (f32::from(weapon.reload_time) * 0.3).trunc() as u16;
        } else {
            weapon.clip_out_time = 0;
            weapon.clip_in_time = 0;
        }

        weapon.timeout = match weapon.bullet_style {
            2 | 9 => GRENADE_TIMEOUT,
            5 => FLAMER_TIMEOUT,
            6 | 11 => MELEE_TIMEOUT,
            14 => M2BULLET_TIMEOUT,
            _ => BULLET_TIMEOUT,
        };

        if kind == WeaponKind::M79 {
            weapon.ammo_count = 0;
        }

        weapon
    }
}
