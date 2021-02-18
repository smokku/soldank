use super::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BulletStyle {
    Bullet = 1,
    FragGrenade = 2,
    GaugeBullet = 3,
    M79Grenade = 4,
    Flame = 5,
    Fist = 6,
    Arrow = 7,
    FlameArrow = 8,
    ClusterGrenade = 9,
    Cluster = 10,
    Blade = 11, // used for knife and chainsaw
    LAWMissile = 12,
    ThrownKnife = 13,
    M2Bullet = 14,
}

#[derive(Debug, Copy, Clone)]
pub struct BulletParams {
    pub style: BulletStyle,
    pub weapon: WeaponKind,
    pub position: Vec2,
    pub velocity: Vec2,
    pub timeout: i16,
    pub hit_multiply: f32,
    pub team: Team,
    pub sprite: Option<gfx::Weapon>,
}

#[derive(Debug, Copy, Clone)]
pub struct Bullet {
    pub active: bool,
    pub style: BulletStyle,
    pub weapon: WeaponKind,
    pub team: Team,
    pub particle: Particle,
    pub initial_pos: Vec2,
    pub velocity_prev: Vec2,
    pub timeout: i16,
    pub timeout_prev: i16,
    pub timeout_real: f32,
    pub hit_multiply: f32,
    pub hit_multiply_prev: f32,
    pub degrade_count: usize,
    pub sprite: Option<gfx::Weapon>,
}

impl Default for BulletStyle {
    fn default() -> BulletStyle {
        BulletStyle::Bullet
    }
}

impl Bullet {
    pub fn new(params: &BulletParams) -> Bullet {
        let particle = Particle {
            active: true,
            pos: params.position,
            old_pos: params.position,
            velocity: params.velocity,
            one_over_mass: 1.0,
            timestep: 1.0,
            gravity: constants::GRAV * 2.25,
            e_damping: 0.99,
            ..Default::default()
        };

        Bullet {
            active: true,
            style: params.style,
            weapon: params.weapon,
            team: params.team,
            particle,
            initial_pos: params.position,
            velocity_prev: params.velocity,
            timeout: params.timeout,
            timeout_prev: params.timeout,
            timeout_real: params.timeout as f32,
            hit_multiply: params.hit_multiply,
            hit_multiply_prev: params.hit_multiply,
            degrade_count: 0,
            sprite: params.sprite,
        }
    }

    pub fn update(&mut self, map: &MapFile) {
        self.velocity_prev = self.particle.velocity;
        self.particle.euler();

        if let Some((pos, _poly)) = self.map_collision(map) {
            self.particle.pos = pos;
            self.active = false;
        }

        self.timeout_prev = self.timeout;
        self.timeout -= 1;

        if self.timeout == 0 {
            self.active = false;
        } else if self.degrade_count < 2 && self.timeout % 6 == 0 {
            let except = [
                WeaponKind::Barrett,
                WeaponKind::M79,
                WeaponKind::Knife,
                WeaponKind::LAW,
            ];

            if !except.contains(&self.weapon) {
                let dist2 = (self.particle.pos - self.initial_pos).length_squared();
                let degrade_dists2 = [500.0 * 500.0, 900.0 * 900.0];

                if dist2 > degrade_dists2[self.degrade_count] {
                    self.hit_multiply_prev = self.hit_multiply;
                    self.hit_multiply *= 0.5;
                    self.degrade_count += 1;
                }
            }
        }

        let (x, y) = self.particle.pos.into();

        if f32::max(x.abs(), y.abs()) > (map.sectors_num * map.sectors_division - 10) as f32 {
            self.active = false;
        }
    }

    pub fn map_collision(&self, map: &MapFile) -> Option<(Vec2, usize)> {
        let a = self.particle.old_pos;
        let b = self.particle.pos;

        let delta = b - a;
        let steps = f32::ceil(delta.length() / 2.5) as i32;

        for i in 0..steps + 1 {
            let (x, y) = lerp(a, b, i as f32 / steps as f32).into();

            for p in map.sector_polys(vec2(x, y)) {
                let p = (*p - 1) as usize;
                let poly = &map.polygons[p];

                if poly.bullet_collides(self.team) && map.point_in_poly_edges(x, y, p as i32) {
                    return Some((vec2(x, y), p));
                }
            }
        }

        None
    }
}
