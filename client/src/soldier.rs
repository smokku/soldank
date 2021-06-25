use super::*;

const SLIDELIMIT: f32 = 0.2;
const SURFACECOEFX: f32 = 0.970;
const SURFACECOEFY: f32 = 0.970;
const CROUCHMOVESURFACECOEFX: f32 = 0.85;
const CROUCHMOVESURFACECOEFY: f32 = 0.97;
const STANDSURFACECOEFX: f32 = 0.00;
const STANDSURFACECOEFY: f32 = 0.00;

const POS_STAND: u8 = 1;
const POS_CROUCH: u8 = 2;
const POS_PRONE: u8 = 3;

const MAX_VELOCITY: f32 = 11.0;
const SOLDIER_COL_RADIUS: f32 = 3.0;

static mut SOLDIER_SKELETON: Option<ParticleSystem> = None;

#[allow(dead_code)]
pub struct Soldier {
    pub active: bool,
    pub dead_meat: bool,
    pub style: u8,
    pub num: usize,
    pub visible: u8,
    pub on_ground: bool,
    pub on_ground_for_law: bool,
    pub on_ground_last_frame: bool,
    pub on_ground_permanent: bool,
    pub direction: i8,
    pub old_direction: i8,
    pub health: f32,
    pub alpha: u8,
    pub jets_count: i32,
    pub jets_count_prev: i32,
    pub wear_helmet: u8,
    pub has_cigar: u8,
    pub vest: f32,
    pub idle_time: i32,
    pub idle_random: i8,
    pub position: u8,
    pub on_fire: u8,
    pub collider_distance: u8,
    pub half_dead: bool,
    pub skeleton: ParticleSystem,
    pub legs_animation: AnimState,
    pub body_animation: AnimState,
    pub control: Control,
    pub active_weapon: usize,
    pub weapons: [Weapon; 3],
    pub fired: u8,
    pub particle: Particle,
}

impl Soldier {
    pub fn initialize(fs: &mut Filesystem, config: &Config) {
        unsafe {
            SOLDIER_SKELETON.replace(ParticleSystem::load_from_file(
                fs,
                "gostek.po",
                4.5,
                1.0,
                1.06 * config.phys.gravity,
                0.0,
                0.9945,
            ));
        }
    }

    pub fn primary_weapon(&self) -> &Weapon {
        &self.weapons[self.active_weapon]
    }

    pub fn secondary_weapon(&self) -> &Weapon {
        &self.weapons[(self.active_weapon + 1) % 2]
    }

    pub fn tertiary_weapon(&self) -> &Weapon {
        &self.weapons[2]
    }

    pub fn switch_weapon(&mut self) {
        let w = (self.active_weapon + 1) % 2;
        self.active_weapon = w;
        self.weapons[w].start_up_time_count = self.weapons[w].start_up_time;
        self.weapons[w].reload_time_prev = self.weapons[w].reload_time_count;
        // burst_count = 0;
    }

    pub fn update_keys(&mut self) {
        self.control.left = mq::is_key_down(mq::KeyCode::A);
        self.control.right = mq::is_key_down(mq::KeyCode::D);
        self.control.up = mq::is_key_down(mq::KeyCode::W);
        self.control.down = mq::is_key_down(mq::KeyCode::S);
        self.control.change = mq::is_key_down(mq::KeyCode::Q);
        self.control.throw = mq::is_key_down(mq::KeyCode::E);
        self.control.drop = mq::is_key_down(mq::KeyCode::F);
        self.control.prone = mq::is_key_down(mq::KeyCode::X);
    }

    pub fn update_mouse_button(&mut self) {
        self.control.fire = mq::is_mouse_button_down(mq::MouseButton::Left);
        self.control.jets = mq::is_mouse_button_down(mq::MouseButton::Right);
    }

    pub fn new(spawn: &MapSpawnpoint, config: &Config) -> Soldier {
        let particle = Particle {
            active: true,
            pos: vec2(spawn.x as f32, spawn.y as f32),
            old_pos: vec2(spawn.x as f32, spawn.y as f32),
            one_over_mass: 1.0,
            timestep: 1.0,
            gravity: config.phys.gravity,
            e_damping: 0.99,
            ..Default::default()
        };

        let weapons = [
            Weapon::new(WeaponKind::DesertEagles, false),
            Weapon::new(WeaponKind::Chainsaw, false),
            Weapon::new(WeaponKind::FragGrenade, false),
        ];

        Soldier {
            active: true,
            dead_meat: false,
            style: 0,
            num: 1,
            visible: 1,
            on_ground: false,
            on_ground_for_law: false,
            on_ground_last_frame: false,
            on_ground_permanent: false,
            direction: 1,
            old_direction: 1,
            health: 150.0,
            alpha: 255,
            jets_count: 0,
            jets_count_prev: 0,
            wear_helmet: 0,
            has_cigar: 1,
            vest: 0.0,
            idle_time: 0,
            idle_random: 0,
            position: 0,
            on_fire: 0,
            collider_distance: 255,
            half_dead: false,
            skeleton: unsafe { SOLDIER_SKELETON.as_ref().unwrap() }.clone(),
            legs_animation: AnimState::new(Anim::Stand),
            body_animation: AnimState::new(Anim::Stand),
            control: Default::default(),
            active_weapon: 0,
            weapons,
            fired: 0,
            particle,
        }
    }

    pub fn legs_apply_animation(&mut self, id: Anim, frame: usize) {
        if !self.legs_animation.is_any(&[Anim::Prone, Anim::ProneMove])
            && self.legs_animation.id != id
        {
            self.legs_animation = AnimState::new(id);
            self.legs_animation.frame = frame;
        }
    }

    pub fn body_apply_animation(&mut self, id: Anim, frame: usize) {
        if self.body_animation.id != id {
            self.body_animation = AnimState::new(id);
            self.body_animation.frame = frame;
        }
    }

    pub fn handle_special_polytypes(&mut self, map: &MapFile, polytype: PolyType, _pos: Vec2) {
        if polytype == PolyType::Deadly
            || polytype == PolyType::BloodyDeadly
            || polytype == PolyType::Explosive
        {
            self.particle.pos = vec2(map.spawnpoints[0].x as f32, map.spawnpoints[0].y as f32);
        }
    }

    pub fn update(&mut self, resources: &Resources) {
        let map = &*resources.get::<MapFile>().unwrap();
        let config = &*resources.get::<Config>().unwrap();

        let mut body_y = 0.0;
        let mut arm_s;

        self.particle.euler();
        self.control(resources);

        *self.skeleton.old_pos_mut(21) = self.skeleton.pos(21);
        *self.skeleton.old_pos_mut(23) = self.skeleton.pos(23);
        // *self.skeleton.old_pos_mut(25) = self.skeleton.pos(25);
        *self.skeleton.pos_mut(21) = self.skeleton.pos(9);
        *self.skeleton.pos_mut(23) = self.skeleton.pos(12);
        // *self.skeleton.pos_mut(25) = self.skeleton.pos(5);

        if !self.dead_meat {
            *self.skeleton.pos_mut(21) = self.skeleton.pos(21) + self.particle.velocity;
            *self.skeleton.pos_mut(23) = self.skeleton.pos(23) + self.particle.velocity;
            // *self.skeleton.pos_mut(25) = self.skeleton.pos(25) + self.particle.velocity;
        }

        match self.position {
            POS_STAND => body_y = 8.0,
            POS_CROUCH => body_y = 9.0,
            POS_PRONE => {
                if self.body_animation.id == Anim::Prone {
                    if self.body_animation.frame > 9 {
                        body_y = -2.0
                    } else {
                        body_y = 14.0 - self.body_animation.frame as f32;
                    }
                } else {
                    body_y = 9.0;
                }

                if self.body_animation.id == Anim::ProneMove {
                    body_y = 0.0;
                }
            }
            _ => {}
        }

        if self.body_animation.id == Anim::GetUp {
            if self.body_animation.frame > 18 {
                body_y = 8.0;
            } else {
                body_y = 4.0;
            }
        }

        if self.control.mouse_aim_x as f32 >= self.particle.pos.x {
            self.direction = 1;
        } else {
            self.direction = -1;
        }

        for i in 1..21 {
            if self.skeleton.active(i) && !self.dead_meat {
                let mut pos = Vec2::ZERO;
                *self.skeleton.old_pos_mut(i) = self.skeleton.pos(i);

                if !self.half_dead && ((i >= 1 && i <= 6) || (i == 17) || (i == 18)) {
                    let anim_pos = self.legs_animation.pos(i);
                    pos.x = self.particle.pos.x + anim_pos.x * f32::from(self.direction);
                    pos.y = self.particle.pos.y + anim_pos.y;
                }

                if i >= 7 && i <= 16 || i == 19 || i == 20 {
                    let anim_pos = self.body_animation.pos(i);
                    pos.x = self.particle.pos.x + anim_pos.x * f32::from(self.direction);
                    pos.y = self.particle.pos.y + anim_pos.y;

                    if self.half_dead {
                        pos.y += 9.0;
                    } else {
                        pos.y += self.skeleton.pos(6).y - (self.particle.pos.y - body_y);
                    };
                }

                *self.skeleton.pos_mut(i) = pos;
            }
        }

        let aim = vec2(
            self.control.mouse_aim_x as f32,
            self.control.mouse_aim_y as f32,
        );

        if !self.dead_meat {
            let pos = self.skeleton.pos(9);
            let r_norm = 0.1 * vec2normalize(self.skeleton.pos(12) - aim);
            let dir = f32::from(self.direction);

            *self.skeleton.pos_mut(12) = pos + vec2(-dir * r_norm.y, dir * r_norm.x);
            *self.skeleton.pos_mut(23) = pos + vec2(-dir * r_norm.y, dir * r_norm.x) * 50.0;
        }

        let not_aiming_anims = [
            Anim::Reload,
            Anim::ReloadBow,
            Anim::ClipIn,
            Anim::ClipOut,
            Anim::SlideBack,
            Anim::Change,
            Anim::ThrowWeapon,
            Anim::Punch,
            Anim::Roll,
            Anim::RollBack,
            Anim::Cigar,
            Anim::Match,
            Anim::Smoke,
            Anim::Wipe,
            Anim::TakeOff,
            Anim::Groin,
            Anim::Piss,
            Anim::Mercy,
            Anim::Mercy2,
            Anim::Victory,
            Anim::Own,
            Anim::Melee,
        ];

        if self.body_animation.id == Anim::Throw {
            arm_s = -5.00;
        } else {
            arm_s = -7.00;
        }

        if !self.body_animation.is_any(&not_aiming_anims) {
            let r_norm = arm_s * vec2normalize(self.skeleton.pos(15) - aim);
            *self.skeleton.pos_mut(15) = self.skeleton.pos(16) + r_norm;
        }

        if self.body_animation.id == Anim::Throw {
            arm_s = -6.00;
        } else {
            arm_s = -8.00;
        }

        if !self.body_animation.is_any(&not_aiming_anims) {
            let r_norm = arm_s * vec2normalize(self.skeleton.pos(19) - aim);
            *self.skeleton.pos_mut(19) = self.skeleton.pos(16) - vec2(0.0, 4.0) + r_norm;
        }

        for i in 1..21 {
            if (self.dead_meat || self.half_dead) && (i < 17) && (i != 7) && (i != 8) {
                let (x, y) = self.particle.pos.into();
                self.on_ground = self.check_skeleton_map_collision(map, i, x, y);
            }
        }

        if !self.dead_meat {
            self.body_animation.do_animation();
            self.legs_animation.do_animation();

            self.on_ground = false;

            let (x, y) = self.particle.pos.into();
            self.check_map_collision(map, config, x - 3.5, y - 12.0, 1);

            let (x, y) = self.particle.pos.into();
            self.check_map_collision(map, config, x + 3.5, y - 12.0, 1);

            body_y = 0.0;
            arm_s = 0.0;

            // Walking either left or right (though only one can be active at once)
            if self.control.left ^ self.control.right {
                if self.control.left ^ (self.direction == 1) {
                    // WRONG
                    arm_s = 0.25;
                } else {
                    body_y = 0.25;
                }
            }
            // If a leg is inside a polygon, caused by the modification of ArmS and
            // BodyY, this is there to not lose contact to ground on slope polygons
            if body_y == 0.0 {
                //let leg_vector = vec2(
                //  self.particle.pos.x + 2.0,
                //  self.particle.pos.y + 1.9,
                //);
                //    if Map.RayCast(LegVector, LegVector, LegDistance, 10) {
                body_y = 0.25;
                // }
            }
            if arm_s == 0.0 {
                //let leg_vector = vec2(
                //  self.particle.pos.x - 2.0,
                //  self.particle.pos.y + 1.9,
                //);
                //    if Map.RayCast(LegVector, LegVector, LegDistance, 10) {
                arm_s = 0.25;
                // }
            }

            let (x, y) = self.particle.pos.into();
            self.on_ground = self.check_map_collision(map, config, x + 2.0, y + 2.0 - body_y, 0);

            let (x, y) = self.particle.pos.into();
            self.on_ground |= self.check_map_collision(map, config, x - 2.0, y + 2.0 - arm_s, 0);

            let (x, y) = self.particle.pos.into();
            let grounded = self.on_ground;
            self.on_ground_for_law = self.check_radius_map_collision(map, x, y - 1.0, grounded);

            let (x, y) = self.particle.pos.into();
            let grounded = self.on_ground || self.on_ground_for_law;
            self.on_ground |= self.check_map_vertices_collision(map, x, y, 3.0, grounded);

            if !(self.on_ground ^ self.on_ground_last_frame) {
                self.on_ground_permanent = self.on_ground;
            }

            self.on_ground_last_frame = self.on_ground;

            if (self.jets_count < map.start_jet) && !(self.control.jets) {
                //if self.on_ground
                /* (MainTickCounter mod 2 = 0) */
                {
                    self.jets_count += 1;
                }
            }

            self.alpha = 255;

            self.skeleton.do_verlet_timestep_for(22, 29);
            self.skeleton.do_verlet_timestep_for(24, 30);
        }

        if self.dead_meat {
            self.skeleton.do_verlet_timestep();
            self.particle.pos = self.skeleton.pos(12);
            //CheckSkeletonOutOfBounds;
        }

        if self.particle.velocity.x > MAX_VELOCITY {
            self.particle.velocity.x = MAX_VELOCITY;
        }
        if self.particle.velocity.x < -MAX_VELOCITY {
            self.particle.velocity.x = -MAX_VELOCITY;
        }
        if self.particle.velocity.y > MAX_VELOCITY {
            self.particle.velocity.y = MAX_VELOCITY;
        }
        if self.particle.velocity.y < -MAX_VELOCITY {
            self.particle.velocity.y = MAX_VELOCITY;
        }
    }

    pub fn check_map_collision(
        &mut self,
        map: &MapFile,
        config: &Config,
        x: f32,
        y: f32,
        area: i32,
    ) -> bool {
        let pos = vec2(x, y) + self.particle.velocity;
        let rx = ((pos.x / map.sectors_division as f32).round()) as i32 + 25;
        let ry = ((pos.y / map.sectors_division as f32).round()) as i32 + 25;

        if (rx > 0) && (rx < map.sectors_num + 25) && (ry > 0) && (ry < map.sectors_num + 25) {
            for j in 0..map.sectors_poly[rx as usize][ry as usize].polys.len() {
                let poly = map.sectors_poly[rx as usize][ry as usize].polys[j] as usize - 1;
                let polytype = map.polygons[poly].polytype;

                if polytype != PolyType::NoCollide && polytype != PolyType::OnlyBulletsCollide {
                    let mut polygons = map.polygons[poly];
                    if map.point_in_poly(pos, &mut polygons) {
                        self.handle_special_polytypes(map, polytype, pos);

                        let mut dist = 0.0;
                        let mut k = 0;

                        let mut perp =
                            map.closest_perpendicular(poly as i32, pos, &mut dist, &mut k);

                        let step = perp;

                        perp = vec2normalize(perp);
                        perp *= dist;
                        dist = vec2length(self.particle.velocity);

                        if vec2length(perp) > dist {
                            perp = vec2normalize(perp);
                            perp *= dist;
                        }
                        if (area == 0)
                            || ((area == 1)
                                && ((self.particle.velocity.y < 0.0)
                                    || (self.particle.velocity.x > SLIDELIMIT)
                                    || (self.particle.velocity.x < -SLIDELIMIT)))
                        {
                            self.particle.old_pos = self.particle.pos;
                            self.particle.pos -= perp;
                            if map.polygons[poly].polytype == PolyType::Bouncy {
                                perp = vec2normalize(perp);
                                perp *= map.polygons[poly].bounciness * dist;
                            }
                            self.particle.velocity -= perp;
                        }

                        if area == 0 {
                            if (self.legs_animation.id == Anim::Stand)
                                || (self.legs_animation.id == Anim::Crouch)
                                || (self.legs_animation.id == Anim::Prone)
                                || (self.legs_animation.id == Anim::ProneMove)
                                || (self.legs_animation.id == Anim::GetUp)
                                || (self.legs_animation.id == Anim::Fall)
                                || (self.legs_animation.id == Anim::Mercy)
                                || (self.legs_animation.id == Anim::Mercy2)
                                || (self.legs_animation.id == Anim::Own)
                            {
                                if (self.particle.velocity.x < SLIDELIMIT)
                                    && (self.particle.velocity.x > -SLIDELIMIT)
                                    && (step.y > SLIDELIMIT)
                                {
                                    self.particle.pos = self.particle.old_pos;
                                    self.particle.force.y -= config.phys.gravity;
                                }

                                if (step.y > SLIDELIMIT)
                                    && (polytype != PolyType::Ice)
                                    && (polytype != PolyType::Bouncy)
                                {
                                    if (self.legs_animation.id == Anim::Stand)
                                        || (self.legs_animation.id == Anim::Fall)
                                        || (self.legs_animation.id == Anim::Crouch)
                                    {
                                        self.particle.velocity.x *= STANDSURFACECOEFX;
                                        self.particle.velocity.y *= STANDSURFACECOEFY;

                                        self.particle.force.x -= self.particle.velocity.x;
                                    } else if self.legs_animation.id == Anim::Prone {
                                        if self.legs_animation.frame > 24 {
                                            if !(self.control.down
                                                && (self.control.left || self.control.right))
                                            {
                                                self.particle.velocity.x *= STANDSURFACECOEFX;
                                                self.particle.velocity.y *= STANDSURFACECOEFY;

                                                self.particle.force.x -= self.particle.velocity.x;
                                            }
                                        } else {
                                            self.particle.velocity.x *= SURFACECOEFX;
                                            self.particle.velocity.y *= SURFACECOEFY;
                                        }
                                    } else if self.legs_animation.id == Anim::GetUp {
                                        self.particle.velocity.x *= SURFACECOEFX;
                                        self.particle.velocity.y *= SURFACECOEFY;
                                    } else if self.legs_animation.id == Anim::ProneMove {
                                        self.particle.velocity.x *= STANDSURFACECOEFX;
                                        self.particle.velocity.y *= STANDSURFACECOEFY;
                                    }
                                }
                            } else if (self.legs_animation.id == Anim::CrouchRun)
                                || (self.legs_animation.id == Anim::CrouchRunBack)
                            {
                                self.particle.velocity.x *= CROUCHMOVESURFACECOEFX;
                                self.particle.velocity.y *= CROUCHMOVESURFACECOEFY;
                            } else {
                                self.particle.velocity.x *= SURFACECOEFX;
                                self.particle.velocity.y *= SURFACECOEFY;
                            }
                        }

                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn check_map_vertices_collision(
        &mut self,
        map: &MapFile,
        x: f32,
        y: f32,
        r: f32,
        has_collided: bool,
    ) -> bool {
        let pos = vec2(x, y) + self.particle.velocity;
        let rx = ((pos.x / map.sectors_division as f32).round()) as i32 + 25;
        let ry = ((pos.y / map.sectors_division as f32).round()) as i32 + 25;

        if (rx > 0) && (rx < map.sectors_num + 25) && (ry > 0) && (ry < map.sectors_num + 25) {
            for j in 0..map.sectors_poly[rx as usize][ry as usize].polys.len() {
                let poly = map.sectors_poly[rx as usize][ry as usize].polys[j] as usize - 1;
                let polytype = map.polygons[poly].polytype;

                if polytype != PolyType::NoCollide && polytype != PolyType::OnlyBulletsCollide {
                    for i in 0..3 {
                        let vert = vec2(
                            map.polygons[poly].vertices[i].x,
                            map.polygons[poly].vertices[i].y,
                        );

                        let dist = distance(vert, pos);
                        if dist < r {
                            if !has_collided {
                                self.handle_special_polytypes(map, polytype, pos);
                            }
                            let mut dir = pos - vert;
                            dir = vec2normalize(dir);
                            self.particle.pos += dir;
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn check_radius_map_collision(
        &mut self,
        map: &MapFile,
        x: f32,
        y: f32,
        has_collided: bool,
    ) -> bool {
        let mut s_pos = vec2(x, y - 3.0);

        let mut det_acc = vec2length(self.particle.velocity).trunc() as i32;
        if det_acc == 0 {
            det_acc = 1;
        }

        let step = self.particle.velocity * (1 / det_acc) as f32;

        for _z in 0..det_acc {
            s_pos += step;

            let rx = ((s_pos.x / map.sectors_division as f32).round()) as i32 + 25;
            let ry = ((s_pos.y / map.sectors_division as f32).round()) as i32 + 25;

            if (rx > 0) && (rx < map.sectors_num + 25) && (ry > 0) && (ry < map.sectors_num + 25) {
                for j in 0..map.sectors_poly[rx as usize][ry as usize].polys.len() {
                    let poly = map.sectors_poly[rx as usize][ry as usize].polys[j] as usize - 1;
                    let polytype = map.polygons[poly].polytype;

                    if polytype != PolyType::NoCollide && polytype != PolyType::OnlyBulletsCollide {
                        for k in 0..2 {
                            let mut norm = map.perps[poly][k];
                            norm *= -SOLDIER_COL_RADIUS;

                            let pos = s_pos + norm;

                            if map.point_in_poly_edges(pos.x, pos.y, poly as i32) {
                                if !has_collided {
                                    self.handle_special_polytypes(map, polytype, pos);
                                }
                                let mut d = 0.0;
                                let mut b = 0;
                                let mut perp =
                                    map.closest_perpendicular(poly as i32, pos, &mut d, &mut b);

                                let mut p1 = vec2(0.0, 0.0);
                                let mut p2 = vec2(0.0, 0.0);
                                match b {
                                    1 => {
                                        p1 = vec2(
                                            map.polygons[poly].vertices[0].x,
                                            map.polygons[poly].vertices[0].y,
                                        );
                                        p2 = vec2(
                                            map.polygons[poly].vertices[1].x,
                                            map.polygons[poly].vertices[1].y,
                                        );
                                    }
                                    2 => {
                                        p1 = vec2(
                                            map.polygons[poly].vertices[1].x,
                                            map.polygons[poly].vertices[1].y,
                                        );
                                        p2 = vec2(
                                            map.polygons[poly].vertices[2].x,
                                            map.polygons[poly].vertices[2].y,
                                        );
                                    }
                                    3 => {
                                        p1 = vec2(
                                            map.polygons[poly].vertices[2].x,
                                            map.polygons[poly].vertices[2].y,
                                        );
                                        p2 = vec2(
                                            map.polygons[poly].vertices[0].x,
                                            map.polygons[poly].vertices[0].y,
                                        );
                                    }
                                    _ => {}
                                }

                                let p3 = pos;
                                let d = point_line_distance(p1, p2, p3);
                                perp *= d;

                                self.particle.pos = self.particle.old_pos;
                                self.particle.velocity = self.particle.force - perp;

                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    pub fn check_skeleton_map_collision(
        &mut self,
        map: &MapFile,
        i: usize,
        x: f32,
        y: f32,
    ) -> bool {
        let mut result = false;
        let pos = vec2(x - 1.0, y + 4.0);
        let rx = ((pos.x / map.sectors_division as f32).round()) as i32 + 25;
        let ry = ((pos.y / map.sectors_division as f32).round()) as i32 + 25;

        if (rx > 0) && (rx < map.sectors_num + 25) && (ry > 0) && (ry < map.sectors_num + 25) {
            for j in 0..map.sectors_poly[rx as usize][ry as usize].polys.len() {
                let poly = map.sectors_poly[rx as usize][ry as usize].polys[j] - 1;

                if map.point_in_poly_edges(pos.x, pos.y, i32::from(poly)) {
                    let mut dist = 0.0;
                    let mut b = 0;
                    let mut perp =
                        map.closest_perpendicular(i32::from(poly), pos, &mut dist, &mut b);
                    perp = vec2normalize(perp);
                    perp *= dist;

                    *self.skeleton.pos_mut(i) = self.skeleton.old_pos(i) - perp;
                    result = true;
                }
            }
        }

        if result {
            let pos = vec2(x, y + 1.0);
            let rx = ((pos.x / map.sectors_division as f32).round()) as i32 + 25;
            let ry = ((pos.y / map.sectors_division as f32).round()) as i32 + 25;

            if (rx > 0) && (rx < map.sectors_num + 25) && (ry > 0) && (ry < map.sectors_num + 25) {
                for j in 0..map.sectors_poly[rx as usize][ry as usize].polys.len() {
                    let poly = map.sectors_poly[rx as usize][ry as usize].polys[j] - 1;
                    //if (Map.PolyType[poly] <> POLY_TYPE_DOESNT) and (Map.PolyType[poly] <> POLY_TYPE_ONLY_BULLETS) then
                    if map.point_in_poly_edges(pos.x, pos.y, i32::from(poly)) {
                        let mut dist = 0.0;
                        let mut b = 0;
                        let mut perp =
                            map.closest_perpendicular(i32::from(poly), pos, &mut dist, &mut b);
                        perp = vec2normalize(perp);
                        perp *= dist;

                        *self.skeleton.pos_mut(i) = self.skeleton.old_pos(i) - perp;
                        result = true;
                    }
                }
            }
        }

        result
    }

    pub fn fire(&self, emitter: &mut Vec<EmitterItem>) {
        let weapon = self.primary_weapon();

        let dir = {
            if weapon.bullet_style == BulletStyle::Blade
                || self.body_animation.id == Anim::Mercy
                || self.body_animation.id == Anim::Mercy2
            {
                vec2normalize(self.skeleton.pos(15) - self.skeleton.pos(16))
            } else {
                let aim_x = self.control.mouse_aim_x as f32;
                let aim_y = self.control.mouse_aim_y as f32;
                vec2normalize(vec2(aim_x, aim_y) - self.skeleton.pos(15))
            }
        };

        let pos = self.skeleton.pos(15) + dir * 4.0 - vec2(0.0, 2.0);
        let bullet_velocity = dir * weapon.speed;
        let inherited_velocity = self.particle.velocity * weapon.inherited_velocity;

        let mut params = BulletParams {
            style: weapon.bullet_style,
            weapon: weapon.kind,
            position: pos,
            velocity: bullet_velocity + inherited_velocity,
            timeout: weapon.timeout as i16,
            hit_multiply: weapon.hit_multiply,
            team: Team::None,
            sprite: weapon.bullet_sprite,
        };

        match weapon.kind {
            WeaponKind::DesertEagles => {
                emitter.push(EmitterItem::Bullet(params));

                let signx = iif!(dir.x > 0.0, 1.0, iif!(dir.x < 0.0, -1.0, 0.0));
                let signy = iif!(dir.x > 0.0, 1.0, iif!(dir.x < 0.0, -1.0, 0.0));

                params.position += vec2(-signx * dir.y, signy * dir.x) * 3.0;
                emitter.push(EmitterItem::Bullet(params));
            }
            WeaponKind::Spas12 => {}
            WeaponKind::Flamer => {}
            WeaponKind::NoWeapon => {}
            WeaponKind::Knife => {}
            WeaponKind::Chainsaw => {}
            WeaponKind::LAW => {}
            _ => emitter.push(EmitterItem::Bullet(params)),
        };
    }
}
