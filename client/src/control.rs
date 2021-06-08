use super::*;

const POS_STAND: u8 = 1;
const POS_CROUCH: u8 = 2;
const POS_PRONE: u8 = 3;

const RUNSPEED: f32 = 0.118;
const RUNSPEEDUP: f32 = RUNSPEED / 6.0;
const FLYSPEED: f32 = 0.03;
const JUMPSPEED: f32 = 0.66;
const CROUCHRUNSPEED: f32 = RUNSPEED / 0.6;
const PRONESPEED: f32 = RUNSPEED * 4.0;
const ROLLSPEED: f32 = RUNSPEED / 1.2;
const JUMPDIRSPEED: f32 = 0.30;
const JETSPEED: f32 = 0.10;
const SECOND: i32 = 60;

const DEFAULT_IDLETIME: i32 = SECOND * 8;

#[derive(Default, Debug)]
pub struct Control {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub fire: bool,
    pub jets: bool,
    pub grenade: bool,
    pub change: bool,
    pub throw: bool,
    pub drop: bool,
    pub reload: bool,
    pub prone: bool,
    pub flag_throw: bool,
    pub mouse_aim_x: i32,
    pub mouse_aim_y: i32,
    pub mouse_dist: i32,
    pub was_running_left: bool,
    pub was_jumping: bool,
    pub was_throwing_weapon: bool,
    pub was_changing_weapon: bool,
    pub was_throwing_grenade: bool,
    pub was_reloading_weapon: bool,
}

impl Soldier {
    pub fn control(&mut self, resources: &resources::Resources) {
        let state = resources.get::<MainState>().unwrap();
        let config = resources.get::<Config>().unwrap();
        let emitter = &mut *resources.get_mut::<Vec<EmitterItem>>().unwrap();

        let mut player_pressed_left_right = false;

        if self.legs_animation.speed < 1 {
            self.legs_animation.speed = 1;
        }

        if self.body_animation.speed < 1 {
            self.body_animation.speed = 1;
        }

        self.control.mouse_aim_x =
            (state.mouse.x - state.game_width as f32 / 2.0 + state.camera.x).round() as i32;
        self.control.mouse_aim_y =
            (state.mouse.y - state.game_height as f32 / 2.0 + state.camera.y).round() as i32;

        let (mut cleft, mut cright) = (self.control.left, self.control.right);

        // If both left and right directions are pressed, then decide which direction to go in
        if cleft && cright {
            // Remember that both directions were pressed, as it's useful for some moves
            player_pressed_left_right = true;

            if self.control.was_jumping {
                // If jumping, keep going in the old direction
                if self.control.was_running_left {
                    cright = false;
                } else {
                    cleft = false;
                }
            } else {
                // If not jumping, instead go in the new direction
                if self.control.was_running_left {
                    cleft = false;
                } else {
                    cright = false;
                }
            }
        } else {
            self.control.was_running_left = cleft;
            self.control.was_jumping = self.control.up;
        }

        let conflicting_keys_pressed =
            |c: &Control| (c.grenade as u8 + c.change as u8 + c.throw as u8 + c.reload as u8) > 1;

        // Handle simultaneous key presses that would conflict
        if conflicting_keys_pressed(&self.control) {
            // At least two buttons pressed, so deactivate any previous one
            if self.control.was_throwing_grenade {
                self.control.grenade = false;
            } else if self.control.was_changing_weapon {
                self.control.change = false;
            } else if self.control.was_throwing_weapon {
                self.control.throw = false;
            } else if self.control.was_reloading_weapon {
                self.control.reload = false;
            }

            // If simultaneously pressing two or more new buttons, then deactivate them in order
            // of least preference
            while conflicting_keys_pressed(&self.control) {
                if self.control.reload {
                    self.control.reload = false;
                } else if self.control.change {
                    self.control.change = false;
                } else if self.control.throw {
                    self.control.throw = false;
                } else if self.control.grenade {
                    self.control.grenade = false;
                }
            }
        } else {
            self.control.was_throwing_grenade = self.control.grenade;
            self.control.was_changing_weapon = self.control.change;
            self.control.was_throwing_weapon = self.control.throw;
            self.control.was_reloading_weapon = self.control.reload;
        }

        if self.dead_meat {
            self.control.free_controls();
        }

        //self.fired = 0;
        self.control.mouse_aim_x =
            (self.control.mouse_aim_x as f32 + self.particle.velocity.x).round() as i32;
        self.control.mouse_aim_y =
            (self.control.mouse_aim_y as f32 + self.particle.velocity.y).round() as i32;

        if self.control.jets
            && (((self.legs_animation.id == Anim::JumpSide)
                && (((self.direction == -1) && cright)
                    || ((self.direction == 1) && cleft)
                    || player_pressed_left_right))
                || ((self.legs_animation.id == Anim::RollBack) && self.control.up))
        {
            self.body_apply_animation(Anim::RollBack, 1);
            self.legs_apply_animation(Anim::RollBack, 1);
        } else if self.control.jets && (self.jets_count > 0) {
            if self.on_ground {
                self.particle.force.y = -2.5
                    * iif!(
                        config.phys.gravity > 0.05,
                        JETSPEED,
                        config.phys.gravity * 2.0
                    );
            } else if self.position != POS_PRONE {
                self.particle.force.y -= iif!(
                    config.phys.gravity > 0.05,
                    JETSPEED,
                    config.phys.gravity * 2.0
                );
            } else {
                self.particle.force.x += f32::from(self.direction)
                    * iif!(
                        config.phys.gravity > 0.05,
                        JETSPEED / 2.0,
                        config.phys.gravity
                    );
            }

            if (self.legs_animation.id != Anim::GetUp)
                && (self.body_animation.id != Anim::Roll)
                && (self.body_animation.id != Anim::RollBack)
            {
                self.legs_apply_animation(Anim::Fall, 1);
            }

            self.jets_count -= 1;
        }

        // FIRE!!!!
        if self.primary_weapon().kind == WeaponKind::Chainsaw
            || (self.body_animation.id != Anim::Roll)
                && (self.body_animation.id != Anim::RollBack)
                && (self.body_animation.id != Anim::Melee)
                && (self.body_animation.id != Anim::Change)
        {
            if ((self.body_animation.id == Anim::HandsUpAim) && (self.body_animation.frame == 11))
                || (self.body_animation.id != Anim::HandsUpAim)
            {
                if self.control.fire
                // and (SpriteC.CeaseFireCounter < 0) */
                {
                    if self.primary_weapon().kind == WeaponKind::NoWeapon
                        || self.primary_weapon().kind == WeaponKind::Knife
                    {
                        self.body_apply_animation(Anim::Punch, 1);
                    } else {
                        self.fire(emitter);
                        self.control.fire = false;
                    }
                }
            }
        }

        // change weapon animation
        if (self.body_animation.id != Anim::Roll) && (self.body_animation.id != Anim::RollBack) {
            if self.control.change {
                self.body_apply_animation(Anim::Change, 1);
            }
        }

        // change weapon
        if self.body_animation.id == Anim::Change {
            if self.body_animation.frame == 2 {
                // TODO: play sound
                self.body_animation.frame += 1;
            } else if self.body_animation.frame == 25 {
                self.switch_weapon();
            } else if (self.body_animation.frame == Anim::Change.num_frames())
                && (self.primary_weapon().ammo_count == 0)
            {
                self.body_apply_animation(Anim::Stand, 1);
            }
        }

        // throw weapon
        if self.control.drop
            && (self.body_animation.id != Anim::Change || self.body_animation.frame > 25)
            && !self.body_animation.is_any(&[Anim::Roll, Anim::RollBack, Anim::ThrowWeapon])
            // && !flamegod bonus
            && !self.primary_weapon().is_any(
                &[
                    WeaponKind::Bow,
                    WeaponKind::FlameBow,
                    WeaponKind::NoWeapon,
                ]
            )
        {
            self.body_apply_animation(Anim::ThrowWeapon, 1);

            if self.primary_weapon().kind == WeaponKind::Knife {
                self.body_animation.speed = 2;
            }
        }

        // throw knife
        if self.body_animation.id == Anim::ThrowWeapon
            && self.primary_weapon().kind == WeaponKind::Knife
            && (!self.control.drop || self.body_animation.frame == 16)
        {
            let weapon = Weapon::new(WeaponKind::ThrownKnife, false);
            let aim_x = self.control.mouse_aim_x as f32;
            let aim_y = self.control.mouse_aim_y as f32;
            let dir = vec2normalize(vec2(aim_x, aim_y) - self.skeleton.pos(15));
            let frame = self.body_animation.frame as f32;
            let thrown_mul = 1.5 * f32::min(16.0, f32::max(8.0, frame)) / 16.0;
            let bullet_vel = dir * weapon.speed * thrown_mul;
            let inherited_vel = self.particle.velocity * weapon.inherited_velocity;
            let velocity = bullet_vel + inherited_vel;

            emitter.push(EmitterItem::Bullet(BulletParams {
                style: weapon.bullet_style,
                weapon: weapon.kind,
                position: self.skeleton.pos(16) + velocity,
                velocity,
                timeout: weapon.timeout as i16,
                hit_multiply: weapon.hit_multiply,
                team: Team::None,
                sprite: weapon.bullet_sprite,
            }));

            self.control.drop = false;
            self.body_apply_animation(Anim::Stand, 1);
        }

        // Punch!
        if !self.dead_meat {
            if (self.body_animation.id == Anim::Punch) && (self.body_animation.frame == 11) {
                self.body_animation.frame += 1;
            }
        }

        // Buttstock!
        if self.dead_meat {
            if (self.body_animation.id == Anim::Melee) && (self.body_animation.frame == 12) {
                // weapons
            }
        }

        if self.body_animation.id == Anim::Melee && self.body_animation.frame > 20 {
            self.body_apply_animation(Anim::Stand, 1);
        }

        // Prone
        if self.control.prone {
            if (self.legs_animation.id != Anim::GetUp)
                && (self.legs_animation.id != Anim::Prone)
                && (self.legs_animation.id != Anim::ProneMove)
            {
                self.legs_apply_animation(Anim::Prone, 1);
                if (self.body_animation.id != Anim::Reload)
                    && (self.body_animation.id != Anim::Change)
                    && (self.body_animation.id != Anim::ThrowWeapon)
                {
                    self.body_apply_animation(Anim::Prone, 1);
                }
                self.old_direction = self.direction;
                self.control.prone = false;
            }
        }

        // Get up
        if self.position == POS_PRONE {
            if self.control.prone || (self.direction != self.old_direction) {
                if ((self.legs_animation.id == Anim::Prone) && (self.legs_animation.frame > 23))
                    || (self.legs_animation.id == Anim::ProneMove)
                {
                    if self.legs_animation.id != Anim::GetUp {
                        self.legs_animation = AnimState::new(Anim::GetUp);
                        self.legs_animation.frame = 9;
                        self.control.prone = false;
                    }
                    if (self.body_animation.id != Anim::Reload)
                        && (self.body_animation.id != Anim::Change)
                        && (self.body_animation.id != Anim::ThrowWeapon)
                    {
                        self.body_apply_animation(Anim::GetUp, 9);
                    }
                }
            }
        }

        let mut unprone = false;
        // Immediately switch from unprone to jump/sidejump, because the end of the unprone
        // animation can be seen as the "wind up" for the jump
        if (self.legs_animation.id == Anim::GetUp)
            && (self.legs_animation.frame > 23 - (4 - 1))
            && self.on_ground
            && self.control.up
            && (cright || cleft)
        {
            // Set sidejump frame 1 to 4 depending on which unprone frame we're in
            let id = self.legs_animation.frame - (23 - (4 - 1));
            self.legs_apply_animation(Anim::JumpSide, id);
            unprone = true;
        } else if (self.legs_animation.id == Anim::GetUp)
            && (self.legs_animation.frame > 23 - (4 - 1))
            && self.on_ground
            && self.control.up
            && !(cright || cleft)
        {
            // Set jump frame 6 to 9 depending on which unprone frame we're in
            let id = self.legs_animation.frame - (23 - (9 - 1));
            self.legs_apply_animation(Anim::Jump, id);
            unprone = true;
        } else if (self.legs_animation.id == Anim::GetUp) && (self.legs_animation.frame > 23) {
            if cright || cleft {
                if (self.direction == 1) ^ cleft {
                    self.legs_apply_animation(Anim::Run, 1);
                } else {
                    self.legs_apply_animation(Anim::RunBack, 1);
                }
            } else if !self.on_ground && self.control.up {
                self.legs_apply_animation(Anim::Run, 1);
            } else {
                self.legs_apply_animation(Anim::Stand, 1);
            }
            unprone = true;
        }

        if unprone {
            self.position = POS_STAND;

            if (self.body_animation.id != Anim::Reload)
                && (self.body_animation.id != Anim::Change)
                && (self.body_animation.id != Anim::ThrowWeapon)
            {
                self.body_apply_animation(Anim::Stand, 1);
            }
        }

        if true {
            // self.stat == 0 {
            if ((self.body_animation.id == Anim::Stand)
                && (self.legs_animation.id == Anim::Stand)
                && !self.dead_meat
                && (self.idle_time > 0))
                || (self.idle_time > DEFAULT_IDLETIME)
            {
                if self.idle_random >= 0 {
                    self.idle_time -= 1;
                }
            } else {
                self.idle_time = DEFAULT_IDLETIME;
            }

            if self.idle_random == 0 {
                if self.idle_time == 0 {
                    self.body_apply_animation(Anim::Smoke, 1);
                    self.idle_time = DEFAULT_IDLETIME;
                }

                if (self.body_animation.id == Anim::Smoke) && (self.body_animation.frame == 17) {
                    self.body_animation.frame += 1;
                }

                if !self.dead_meat {
                    if (self.idle_time == 1)
                        && (self.body_animation.id != Anim::Smoke)
                        && (self.legs_animation.id == Anim::Stand)
                    {
                        self.idle_time = DEFAULT_IDLETIME;
                        self.idle_random = -1;
                    }
                }
            }

            // *CHEAT*
            if self.legs_animation.speed > 1 {
                if (self.legs_animation.id == Anim::Jump)
                    || (self.legs_animation.id == Anim::JumpSide)
                    || (self.legs_animation.id == Anim::Roll)
                    || (self.legs_animation.id == Anim::RollBack)
                    || (self.legs_animation.id == Anim::Prone)
                    || (self.legs_animation.id == Anim::Run)
                    || (self.legs_animation.id == Anim::RunBack)
                {
                    self.particle.velocity.x /= self.legs_animation.speed as f32;
                    self.particle.velocity.y /= self.legs_animation.speed as f32;
                }

                if self.legs_animation.speed > 2 {
                    if (self.legs_animation.id == Anim::ProneMove)
                        || (self.legs_animation.id == Anim::CrouchRun)
                    {
                        self.particle.velocity.x /= self.legs_animation.speed as f32;
                        self.particle.velocity.y /= self.legs_animation.speed as f32;
                    }
                }
            }

            // TODO: Check if near collider

            // TODO if targetmode > freecontrols
            // End any ongoing idle animations if a key is pressed
            if (self.body_animation.id == Anim::Cigar)
                || (self.body_animation.id == Anim::Match)
                || (self.body_animation.id == Anim::Smoke)
                || (self.body_animation.id == Anim::Wipe)
                || (self.body_animation.id == Anim::Groin)
            {
                if cleft
                    || cright
                    || self.control.up
                    || self.control.down
                    || self.control.fire
                    || self.control.jets
                    || self.control.grenade
                    || self.control.change
                    || self.control.change
                    || self.control.throw
                    || self.control.reload
                    || self.control.prone
                {
                    self.body_animation.frame = self.body_animation.num_frames();
                }
            }

            // make anims out of controls
            // rolling
            if (self.body_animation.id != Anim::TakeOff)
                && (self.body_animation.id != Anim::Piss)
                && (self.body_animation.id != Anim::Mercy)
                && (self.body_animation.id != Anim::Mercy2)
                && (self.body_animation.id != Anim::Victory)
                && (self.body_animation.id != Anim::Own)
            {
                if (self.body_animation.id == Anim::Roll)
                    || (self.body_animation.id == Anim::RollBack)
                {
                    if self.legs_animation.id == Anim::Roll {
                        if self.on_ground {
                            self.particle.force.x = f32::from(self.direction) * ROLLSPEED;
                        } else {
                            self.particle.force.x = f32::from(self.direction) * 2.0 * FLYSPEED;
                        }
                    } else if self.legs_animation.id == Anim::RollBack {
                        if self.on_ground {
                            self.particle.force.x = -f32::from(self.direction) * ROLLSPEED;
                        } else {
                            self.particle.force.x = -f32::from(self.direction) * 2.0 * FLYSPEED;
                        }
                        // if appropriate frames to move
                        if (self.legs_animation.frame > 1) && (self.legs_animation.frame < 8) {
                            if self.control.up {
                                self.particle.force.y -= JUMPDIRSPEED * 1.5;
                                self.particle.force.x *= 0.5;
                                self.particle.velocity.x *= 0.8;
                            }
                        }
                    }
                // downright
                } else if (cright) && (self.control.down) {
                    if self.on_ground {
                        // roll to the side
                        if (self.legs_animation.id == Anim::Run)
                            || (self.legs_animation.id == Anim::RunBack)
                            || (self.legs_animation.id == Anim::Fall)
                            || (self.legs_animation.id == Anim::ProneMove)
                            || ((self.legs_animation.id == Anim::Prone)
                                && (self.legs_animation.frame >= 24))
                        {
                            if (self.legs_animation.id == Anim::ProneMove)
                                || ((self.legs_animation.id == Anim::Prone)
                                    && (self.legs_animation.frame
                                        == self.legs_animation.num_frames()))
                            {
                                self.control.prone = false;
                                self.position = POS_STAND;
                            }

                            if self.direction == 1 {
                                self.body_apply_animation(Anim::Roll, 1);
                                self.legs_animation = AnimState::new(Anim::Roll);
                                self.legs_animation.frame = 1;
                            } else {
                                self.body_apply_animation(Anim::RollBack, 1);
                                self.legs_animation = AnimState::new(Anim::RollBack);
                                self.legs_animation.frame = 1;
                            }
                        } else {
                            if self.direction == 1 {
                                self.legs_apply_animation(Anim::CrouchRun, 1);
                            } else {
                                self.legs_apply_animation(Anim::CrouchRunBack, 1);
                            }
                        }

                        if (self.legs_animation.id == Anim::CrouchRun)
                            || (self.legs_animation.id == Anim::CrouchRunBack)
                        {
                            self.particle.force.x = CROUCHRUNSPEED;
                        } else if (self.legs_animation.id == Anim::Roll)
                            || (self.legs_animation.id == Anim::RollBack)
                        {
                            self.particle.force.x = 2.0 * CROUCHRUNSPEED;
                        }
                    }
                // downleft
                } else if cleft && self.control.down {
                    if self.on_ground {
                        // roll to the side
                        if (self.legs_animation.id == Anim::Run)
                            || (self.legs_animation.id == Anim::RunBack)
                            || (self.legs_animation.id == Anim::Fall)
                            || (self.legs_animation.id == Anim::ProneMove)
                            || ((self.legs_animation.id == Anim::Prone)
                                && (self.legs_animation.frame >= 24))
                        {
                            if (self.legs_animation.id == Anim::ProneMove)
                                || ((self.legs_animation.id == Anim::Prone)
                                    && (self.legs_animation.frame
                                        == self.legs_animation.num_frames()))
                            {
                                self.control.prone = false;
                                self.position = POS_STAND;
                            }

                            if self.direction == 1 {
                                self.body_apply_animation(Anim::RollBack, 1);
                                self.legs_animation = AnimState::new(Anim::RollBack);
                                self.legs_animation.frame = 1;
                            } else {
                                self.body_apply_animation(Anim::Roll, 1);
                                self.legs_animation = AnimState::new(Anim::Roll);
                                self.legs_animation.frame = 1;
                            }
                        } else {
                            if self.direction == 1 {
                                self.legs_apply_animation(Anim::CrouchRunBack, 1);
                            } else {
                                self.legs_apply_animation(Anim::CrouchRun, 1);
                            }
                        }

                        if (self.legs_animation.id == Anim::CrouchRun)
                            || (self.legs_animation.id == Anim::CrouchRunBack)
                        {
                            self.particle.force.x = -CROUCHRUNSPEED;
                        }
                    }
                // Proning
                } else if (self.legs_animation.id == Anim::Prone)
                    || (self.legs_animation.id == Anim::ProneMove)
                    || ((self.legs_animation.id == Anim::GetUp)
                        && (self.body_animation.id != Anim::Throw)
                        && (self.body_animation.id != Anim::Punch))
                {
                    if self.on_ground {
                        if ((self.legs_animation.id == Anim::Prone)
                            && (self.legs_animation.frame > 25))
                            || (self.legs_animation.id == Anim::ProneMove)
                        {
                            if cleft || cright {
                                if (self.legs_animation.frame < 4)
                                    || (self.legs_animation.frame > 14)
                                {
                                    self.particle.force.x = {
                                        if cleft {
                                            -PRONESPEED
                                        } else {
                                            PRONESPEED
                                        }
                                    }
                                }

                                self.legs_apply_animation(Anim::ProneMove, 1);

                                if (self.body_animation.id != Anim::ClipIn)
                                    && (self.body_animation.id != Anim::ClipOut)
                                    && (self.body_animation.id != Anim::SlideBack)
                                    && (self.body_animation.id != Anim::Reload)
                                    && (self.body_animation.id != Anim::Change)
                                    && (self.body_animation.id != Anim::Throw)
                                    && (self.body_animation.id != Anim::ThrowWeapon)
                                {
                                    self.body_apply_animation(Anim::ProneMove, 1);
                                }

                                if self.legs_animation.id != Anim::ProneMove {
                                    self.legs_animation = AnimState::new(Anim::ProneMove);
                                }
                            } else {
                                if self.legs_animation.id != Anim::Prone {
                                    self.legs_animation = AnimState::new(Anim::Prone);
                                }
                                self.legs_animation.frame = 26;
                            }
                        }
                    }
                } else if cright && self.control.up {
                    if self.on_ground {
                        if (self.legs_animation.id == Anim::Run)
                            || (self.legs_animation.id == Anim::RunBack)
                            || (self.legs_animation.id == Anim::Stand)
                            || (self.legs_animation.id == Anim::Crouch)
                            || (self.legs_animation.id == Anim::CrouchRun)
                            || (self.legs_animation.id == Anim::CrouchRunBack)
                        {
                            self.legs_apply_animation(Anim::JumpSide, 1);
                        }

                        if self.legs_animation.frame == self.legs_animation.num_frames() {
                            self.legs_apply_animation(Anim::Run, 1);
                        }
                    } else if (self.legs_animation.id == Anim::Roll)
                        || (self.legs_animation.id == Anim::RollBack)
                    {
                        if self.direction == 1 {
                            self.legs_apply_animation(Anim::Run, 1);
                        } else {
                            self.legs_apply_animation(Anim::RunBack, 1);
                        }
                    }
                    if self.legs_animation.id == Anim::Jump {
                        if self.legs_animation.frame < 10 {
                            self.legs_apply_animation(Anim::JumpSide, 1);
                        }
                    }

                    if self.legs_animation.id == Anim::JumpSide {
                        if (self.legs_animation.frame > 3) && (self.legs_animation.frame < 11) {
                            self.particle.force.x = JUMPDIRSPEED;
                            self.particle.force.y = -JUMPDIRSPEED / 1.2;
                        }
                    }
                } else if cleft && self.control.up {
                    if self.on_ground {
                        if (self.legs_animation.id == Anim::Run)
                            || (self.legs_animation.id == Anim::RunBack)
                            || (self.legs_animation.id == Anim::Stand)
                            || (self.legs_animation.id == Anim::Crouch)
                            || (self.legs_animation.id == Anim::CrouchRun)
                            || (self.legs_animation.id == Anim::CrouchRunBack)
                        {
                            self.legs_apply_animation(Anim::JumpSide, 1);
                        }

                        if self.legs_animation.frame == self.legs_animation.num_frames() {
                            self.legs_apply_animation(Anim::Run, 1);
                        }
                    } else if (self.legs_animation.id == Anim::Roll)
                        || (self.legs_animation.id == Anim::RollBack)
                    {
                        if self.direction == -1 {
                            self.legs_apply_animation(Anim::Run, 1);
                        } else {
                            self.legs_apply_animation(Anim::RunBack, 1);
                        }
                    }

                    if self.legs_animation.id == Anim::Jump {
                        if self.legs_animation.frame < 10 {
                            self.legs_apply_animation(Anim::JumpSide, 1);
                        }
                    }

                    if self.legs_animation.id == Anim::JumpSide {
                        if (self.legs_animation.frame > 3) && (self.legs_animation.frame < 11) {
                            self.particle.force.x = -JUMPDIRSPEED;
                            self.particle.force.y = -JUMPDIRSPEED / 1.2;
                        }
                    }
                } else if self.control.up {
                    if self.on_ground {
                        if self.legs_animation.id != Anim::Jump {
                            self.legs_apply_animation(Anim::Jump, 1);
                        }
                        if self.legs_animation.frame == self.legs_animation.num_frames() {
                            self.legs_apply_animation(Anim::Stand, 1);
                        }
                    }
                    if self.legs_animation.id == Anim::Jump {
                        if (self.legs_animation.frame > 8) && (self.legs_animation.frame < 15) {
                            self.particle.force.y = -JUMPSPEED;
                        }
                        if self.legs_animation.frame == self.legs_animation.num_frames() {
                            self.legs_apply_animation(Anim::Fall, 1);
                        }
                    }
                } else if self.control.down {
                    if self.on_ground {
                        self.legs_apply_animation(Anim::Crouch, 1);
                    }
                } else if cright {
                    if true {
                        // if self.para = 0
                        if self.direction == 1 {
                            self.legs_apply_animation(Anim::Run, 1);
                        } else {
                            self.legs_apply_animation(Anim::RunBack, 1);
                        }
                    }

                    if self.on_ground {
                        self.particle.force.x = RUNSPEED;
                        self.particle.force.y = -RUNSPEEDUP;
                    } else {
                        self.particle.force.x = FLYSPEED;
                    }
                } else if cleft {
                    if true {
                        // if self.para = 0
                        if self.direction == -1 {
                            self.legs_apply_animation(Anim::Run, 1);
                        } else {
                            self.legs_apply_animation(Anim::RunBack, 1);
                        }
                    }

                    if self.on_ground {
                        self.particle.force.x = -RUNSPEED;
                        self.particle.force.y = -RUNSPEEDUP;
                    } else {
                        self.particle.force.x = -FLYSPEED;
                    }
                } else {
                    if self.on_ground {
                        self.legs_apply_animation(Anim::Stand, 1);
                    } else {
                        self.legs_apply_animation(Anim::Fall, 1);
                    }
                }
            }
            // Body animations

            if (self.legs_animation.id == Anim::Roll) && (self.body_animation.id != Anim::Roll) {
                self.body_apply_animation(Anim::Roll, 1)
            }
            if (self.body_animation.id == Anim::Roll) && (self.legs_animation.id != Anim::Roll) {
                self.legs_apply_animation(Anim::Roll, 1)
            }
            if (self.legs_animation.id == Anim::RollBack)
                && (self.body_animation.id != Anim::RollBack)
            {
                self.body_apply_animation(Anim::RollBack, 1)
            }
            if (self.body_animation.id == Anim::RollBack)
                && (self.legs_animation.id != Anim::RollBack)
            {
                self.legs_apply_animation(Anim::RollBack, 1)
            }

            if (self.body_animation.id == Anim::Roll) || (self.body_animation.id == Anim::RollBack)
            {
                if self.legs_animation.frame != self.body_animation.frame {
                    if self.legs_animation.frame > self.body_animation.frame {
                        self.body_animation.frame = self.legs_animation.frame;
                    } else {
                        self.legs_animation.frame = self.body_animation.frame;
                    }
                }
            }

            // Gracefully end a roll animation
            if ((self.body_animation.id == Anim::Roll)
                || (self.body_animation.id == Anim::RollBack))
                && (self.body_animation.frame == self.body_animation.num_frames())
            {
                // Was probably a roll
                if self.on_ground {
                    if self.control.down {
                        if cleft || cright {
                            if self.body_animation.id == Anim::Roll {
                                self.legs_apply_animation(Anim::CrouchRun, 1);
                            } else {
                                self.legs_apply_animation(Anim::CrouchRunBack, 1);
                            }
                        } else {
                            self.legs_apply_animation(Anim::Crouch, 15);
                        }
                    }
                // Was probably a backflip
                } else if (self.body_animation.id == Anim::RollBack) && self.control.up {
                    if cleft || cright {
                        // Run back or forward depending on facing direction and direction key pressed
                        if (self.direction == 1) ^ (cleft) {
                            self.legs_apply_animation(Anim::Run, 1);
                        } else {
                            self.legs_apply_animation(Anim::RunBack, 1);
                        }
                    } else {
                        self.legs_apply_animation(Anim::Fall, 1);
                    }
                // Was probably a roll (that ended mid-air)
                } else if self.control.down {
                    if cleft || cright {
                        if self.body_animation.id == Anim::Roll {
                            self.legs_apply_animation(Anim::CrouchRun, 1);
                        } else {
                            self.legs_apply_animation(Anim::CrouchRunBack, 1);
                        }
                    } else {
                        self.legs_apply_animation(Anim::Crouch, 15);
                    }
                }
                self.body_apply_animation(Anim::Stand, 1);
            }

            if (!self.control.grenade
                && (self.body_animation.id != Anim::Recoil)
                && (self.body_animation.id != Anim::SmallRecoil)
                && (self.body_animation.id != Anim::AimRecoil)
                && (self.body_animation.id != Anim::HandsUpRecoil)
                && (self.body_animation.id != Anim::Shotgun)
                && (self.body_animation.id != Anim::Barret)
                && (self.body_animation.id != Anim::Change)
                && (self.body_animation.id != Anim::ThrowWeapon)
                && (self.body_animation.id != Anim::WeaponNone)
                && (self.body_animation.id != Anim::Punch)
                && (self.body_animation.id != Anim::Roll)
                && (self.body_animation.id != Anim::RollBack)
                && (self.body_animation.id != Anim::ReloadBow)
                && (self.body_animation.id != Anim::Cigar)
                && (self.body_animation.id != Anim::Match)
                && (self.body_animation.id != Anim::Smoke)
                && (self.body_animation.id != Anim::Wipe)
                && (self.body_animation.id != Anim::TakeOff)
                && (self.body_animation.id != Anim::Groin)
                && (self.body_animation.id != Anim::Piss)
                && (self.body_animation.id != Anim::Mercy)
                && (self.body_animation.id != Anim::Mercy2)
                && (self.body_animation.id != Anim::Victory)
                && (self.body_animation.id != Anim::Own)
                && (self.body_animation.id != Anim::Reload)
                && (self.body_animation.id != Anim::Prone)
                && (self.body_animation.id != Anim::GetUp)
                && (self.body_animation.id != Anim::ProneMove)
                && (self.body_animation.id != Anim::Melee))
                || ((self.body_animation.frame == self.body_animation.num_frames())
                    && (self.body_animation.id != Anim::Prone))
            {
                if self.position != POS_PRONE {
                    if self.position == POS_STAND {
                        self.body_apply_animation(Anim::Stand, 1);
                    }

                    if self.position == POS_CROUCH {
                        if self.collider_distance < 255 {
                            if self.body_animation.id == Anim::HandsUpRecoil {
                                self.body_apply_animation(Anim::HandsUpAim, 11);
                            } else {
                                self.body_apply_animation(Anim::HandsUpAim, 1);
                            }
                        } else {
                            if self.body_animation.id == Anim::AimRecoil {
                                self.body_apply_animation(Anim::Aim, 6);
                            } else {
                                self.body_apply_animation(Anim::Aim, 1);
                            }
                        }
                    }
                } else {
                    self.body_apply_animation(Anim::Prone, 26);
                }
            }

            if (self.legs_animation.id == Anim::Crouch)
                || (self.legs_animation.id == Anim::CrouchRun)
                || (self.legs_animation.id == Anim::CrouchRunBack)
            {
                self.position = POS_CROUCH;
            } else {
                self.position = POS_STAND;
            }
            if (self.legs_animation.id == Anim::Prone)
                || (self.legs_animation.id == Anim::ProneMove)
            {
                self.position = POS_PRONE;
            }
        }
    }
}

impl Control {
    pub fn free_controls(&mut self) {
        *self = Default::default();
    }
}
