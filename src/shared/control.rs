use shared::state::MainState;
use shared::sprites::*;

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

impl Sprite {
  pub fn control(&mut self, state: &mut MainState) {
    let mut player_pressed_left_right = false;
    if self.legs_animation.speed < 1 {
      self.legs_animation.speed = 1;
    }
    if self.body_animation.speed < 1 {
      self.body_animation.speed = 1;
    }

    println!(
      "curr_frame: {}, animation_id: {}, body_animation_id: {}, num_frames: {} direction: {}",
      self.legs_animation.curr_frame,
      self.legs_animation.id,
      self.body_animation.id,
      self.legs_animation.num_frames,
      self.direction
    );

    self.control.mouse_aim_x =
      (state.mouse.x - state.game_width as f32 / 2.0 + state.camera.x).round() as i32;
    self.control.mouse_aim_y =
      (state.mouse.y - state.game_height as f32 / 2.0 + state.camera.y).round() as i32;

    // If both left and right directions are pressed, then decide which direction to go in
    if self.control.left && self.control.right {
      // Remember that both directions were pressed, as it's useful for some moves
      player_pressed_left_right = true;

      if self.control.was_jumping {
        // If jumping, keep going in the old direction
        if self.control.was_running_left {
          self.control.right = false;
        } else {
          self.control.left = false;
        }
      } else {
        // If not jumping, instead go in the new direction
        if self.control.was_running_left {
          self.control.left = false;
        } else {
          self.control.right = false;
        }
      }
    } else {
      self.control.was_running_left = self.control.left;
      self.control.was_jumping = self.control.up;
    }

    // Handle simultaneous key presses that would conflict
    if ((self.control.grenade as u8) + self.control.change as u8 + self.control.throw as u8
      + self.control.reload as u8) > 1
    {
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
      // of least prefecence
      while (self.control.grenade as u8 + self.control.change as u8 + self.control.throw as u8
        + self.control.reload as u8) > 1
      {
        if self.control.reload {
          self.control.reload = false;
        }
        if self.control.change {
          self.control.change = false;
        }
        if self.control.throw {
          self.control.throw = false;
        }
        if self.control.grenade {
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
      (self.control.mouse_aim_x as f32 + state.sprite_parts.velocity[self.num].x).round() as i32;
    self.control.mouse_aim_y =
      (self.control.mouse_aim_y as f32 + state.sprite_parts.velocity[self.num].y).round() as i32;

    if self.control.jets
      && (((self.legs_animation.id == state.anims.jump_side.id)
        && (((self.direction == -1) && self.control.right)
          || ((self.direction == 1) && self.control.left) || player_pressed_left_right))
        || ((self.legs_animation.id == state.anims.roll_back.id) && self.control.up))
    {
      self.body_apply_animation(state.anims.roll_back.clone(), 1);
      self.legs_apply_animation(state.anims.roll_back.clone(), 1);
    } else {
      if self.control.jets && (self.jets_count > 0) {
        if self.on_ground {
          state.sprite_parts.forces[self.num].y = -2.5 * {
            if state.gravity > 0.05 {
              JETSPEED
            } else {
              state.gravity * 2.0
            }
          };
        } else {
          if self.position != POS_PRONE {
            state.sprite_parts.forces[self.num].y = state.sprite_parts.forces[self.num].y - {
              if state.gravity > 0.05 {
                JETSPEED
              } else {
                state.gravity * 2.0
              }
            };
          } else {
            state.sprite_parts.forces[self.num].x = state.sprite_parts.forces[self.num].x
              + (f32::from(self.direction) * {
                if state.gravity > 0.05 {
                  JETSPEED
                } else {
                  state.gravity * 2.0
                }
              } / 2.0);
          }
        }

        if (self.legs_animation.id != state.anims.get_up.id)
          && (self.body_animation.id != state.anims.roll.id)
          && (self.body_animation.id != state.anims.roll_back.id)
        {
          self.legs_apply_animation(state.anims.fall.clone(), 1);
        }

        self.jets_count -= 1;

        if (self.jets_count == 1) && self.control.jets {
          self.jets_count = 0;
        }
      }
    }

    // Buttstock!
    if self.dead_meat {
      if (self.body_animation.id == state.anims.melee.id) && (self.body_animation.curr_frame == 12)
      {
        // weapons
      }
    }

    if self.body_animation.id == state.anims.melee.id && self.body_animation.curr_frame > 20 {
      self.body_apply_animation(state.anims.stand.clone(), 1);
    }

    // Prone
    if self.control.prone {
      if (self.legs_animation.id != state.anims.get_up.id)
        && (self.legs_animation.id != state.anims.prone.id)
        && (self.legs_animation.id != state.anims.prone_move.id)
      {
        self.legs_apply_animation(state.anims.prone.clone(), 1);
        if (self.body_animation.id != state.anims.reload.id)
          && (self.body_animation.id != state.anims.change.id)
          && (self.body_animation.id != state.anims.throw_weapon.id)
        {
          self.body_apply_animation(state.anims.prone.clone(), 1);
        }
        self.old_direction = self.direction;
        self.control.prone = false;
      }
    }

    // Get up
    if self.position == POS_PRONE {
      if self.control.prone || (self.direction != self.old_direction) {
        if ((self.legs_animation.id == state.anims.prone.id)
          && (self.legs_animation.curr_frame > 23))
          || (self.legs_animation.id == state.anims.prone_move.id)
        {
          if self.legs_animation.id != state.anims.get_up.id {
            self.legs_animation = state.anims.get_up.clone();
            self.legs_animation.curr_frame = 9;
            self.control.prone = false;
          }
          if (self.body_animation.id != state.anims.reload.id)
            && (self.body_animation.id != state.anims.change.id)
            && (self.body_animation.id != state.anims.throw_weapon.id)
          {
            self.body_apply_animation(state.anims.get_up.clone(), 9);
          }
        }
      }
    }

    let mut unprone = false;
    // Immediately switch from unprone to jump/sidejump, because the end of the unprone
    // animation can be seen as the "wind up" for the jump
    if (self.legs_animation.id == state.anims.get_up.id)
      && (self.legs_animation.curr_frame > 23 - (4 - 1)) && self.on_ground && self.control.up
      && (self.control.right || self.control.left)
    {
      // Set sidejump frame 1 to 4 depending on which unprone frame we're in
      let id = self.legs_animation.curr_frame - (23 - (4 - 1));
      self.legs_apply_animation(state.anims.jump_side.clone(), id);
      unprone = true;
    } else if (self.legs_animation.id == state.anims.get_up.id)
      && (self.legs_animation.curr_frame > 23 - (4 - 1)) && self.on_ground
      && self.control.up && !(self.control.right || self.control.left)
    {
      // Set jump frame 6 to 9 depending on which unprone frame we're in
      let id = self.legs_animation.curr_frame - (23 - (9 - 1));
      self.legs_apply_animation(state.anims.jump.clone(), id);
      unprone = true;
    } else if (self.legs_animation.id == state.anims.get_up.id)
      && (self.legs_animation.curr_frame > 23)
    {
      if self.control.right || self.control.left {
        if (self.direction == 1) ^ self.control.left {
          self.legs_apply_animation(state.anims.run.clone(), 1);
        } else {
          self.legs_apply_animation(state.anims.run_back.clone(), 1);
        }
      } else if !self.on_ground && self.control.up {
        self.legs_apply_animation(state.anims.run.clone(), 1);
      } else {
        self.legs_apply_animation(state.anims.stand.clone(), 1);
      }
      unprone = true;
    }

    if unprone {
      self.position = POS_STAND;

      if (self.body_animation.id != state.anims.reload.id)
        && (self.body_animation.id != state.anims.change.id)
        && (self.body_animation.id != state.anims.throw_weapon.id)
      {
        self.body_apply_animation(state.anims.stand.clone(), 1);
      }
    }

    if true {
      // self.stat == 0 {
      if ((self.body_animation.id == state.anims.stand.id)
        && (self.legs_animation.id == state.anims.stand.id) && !self.dead_meat
        && (self.idle_time > 0)) || (self.idle_time > DEFAULT_IDLETIME)
      {
        if self.idle_random >= 0 {
          self.idle_time -= 1;
        }
      } else {
        self.idle_time = DEFAULT_IDLETIME;
      }

      if self.idle_random == 0 {
        if self.idle_time == 0 {
          self.body_apply_animation(state.anims.smoke.clone(), 1);
          self.idle_time = DEFAULT_IDLETIME;
        }

        if (self.body_animation.id == state.anims.smoke.id)
          && (self.body_animation.curr_frame == 17)
        {
          self.body_animation.curr_frame += 1;
        }

        if !self.dead_meat {
          if (self.idle_time == 1) && (self.body_animation.id != state.anims.smoke.id)
            && (self.legs_animation.id == state.anims.stand.id)
          {
            self.idle_time = DEFAULT_IDLETIME;
            self.idle_random = -1;
          }
        }
      }

      // *CHEAT*
      if self.legs_animation.speed > 1 {
        if (self.legs_animation.id == state.anims.jump.id)
          || (self.legs_animation.id == state.anims.jump_side.id)
          || (self.legs_animation.id == state.anims.roll.id)
          || (self.legs_animation.id == state.anims.roll_back.id)
          || (self.legs_animation.id == state.anims.prone.id)
          || (self.legs_animation.id == state.anims.run.id)
          || (self.legs_animation.id == state.anims.run_back.id)
        {
          state.sprite_parts.velocity[self.num].x /= self.legs_animation.speed as f32;
          state.sprite_parts.velocity[self.num].y /= self.legs_animation.speed as f32;
        }

        if self.legs_animation.speed > 2 {
          if (self.legs_animation.id == state.anims.prone_move.id)
            || (self.legs_animation.id == state.anims.crouch_run.id)
          {
            state.sprite_parts.velocity[self.num].x /= self.legs_animation.speed as f32;
            state.sprite_parts.velocity[self.num].y /= self.legs_animation.speed as f32;
          }
        }
      }

      // TODO: Check if near collider

      // TODO if targetmode > freecontrols
      // End any ongoing idle animations if a key is pressed
      if (self.body_animation.id == state.anims.cigar.id)
        || (self.body_animation.id == state.anims.match_.id)
        || (self.body_animation.id == state.anims.smoke.id)
        || (self.body_animation.id == state.anims.wipe.id)
        || (self.body_animation.id == state.anims.groin.id)
      {
        if self.control.left || self.control.right || self.control.up || self.control.down
          || self.control.fire || self.control.jets || self.control.grenade
          || self.control.change || self.control.change || self.control.throw
          || self.control.reload || self.control.prone
        {
          self.body_animation.curr_frame = self.body_animation.num_frames;
        }
      }

      // make anims out of controls
      // rolling
      if (self.body_animation.id != state.anims.take_off.id)
        && (self.body_animation.id != state.anims.piss.id)
        && (self.body_animation.id != state.anims.mercy.id)
        && (self.body_animation.id != state.anims.mercy2.id)
        && (self.body_animation.id != state.anims.victory.id)
        && (self.body_animation.id != state.anims.own.id)
      {
        if (self.body_animation.id == state.anims.roll.id)
          || (self.body_animation.id == state.anims.roll_back.id)
        {
          if self.legs_animation.id == state.anims.roll.id {
            if self.on_ground {
              state.sprite_parts.forces[self.num].x = f32::from(self.direction) * ROLLSPEED;
            } else {
              state.sprite_parts.forces[self.num].x = f32::from(self.direction) * 2.0 * FLYSPEED;
            }
          } else if self.legs_animation.id == state.anims.roll_back.id {
            if self.on_ground {
              state.sprite_parts.forces[self.num].x = -f32::from(self.direction) * ROLLSPEED;
            } else {
              state.sprite_parts.forces[self.num].x = -f32::from(self.direction) * 2.0 * FLYSPEED;
            }
            // if appropriate frames to move
            if (self.legs_animation.curr_frame > 1) && (self.legs_animation.curr_frame < 8) {
              if self.control.up {
                state.sprite_parts.forces[self.num].y -= JUMPDIRSPEED * 1.5;
                state.sprite_parts.forces[self.num].x *= 0.5;
                state.sprite_parts.velocity[self.num].x *= 0.8;
              }
            }
          }
        // downright
        } else if (self.control.right) && (self.control.down) {
          if self.on_ground {
            // roll to the side
            if (self.legs_animation.id == state.anims.run.id)
              || (self.legs_animation.id == state.anims.run_back.id)
              || (self.legs_animation.id == state.anims.fall.id)
              || (self.legs_animation.id == state.anims.prone_move.id)
              || ((self.legs_animation.id == state.anims.prone.id)
                && (self.legs_animation.curr_frame >= 24))
            {
              if (self.legs_animation.id == state.anims.prone_move.id)
                || ((self.legs_animation.id == state.anims.prone.id)
                  && (self.legs_animation.curr_frame == self.legs_animation.num_frames))
              {
                self.control.prone = false;
                self.position = POS_STAND;
              }

              if self.direction == 1 {
                self.body_apply_animation(state.anims.roll.clone(), 1);
                self.legs_animation = state.anims.roll.clone();
                self.legs_animation.curr_frame = 1;
              } else {
                self.body_apply_animation(state.anims.roll_back.clone(), 1);
                self.legs_animation = state.anims.roll_back.clone();
                self.legs_animation.curr_frame = 1;
              }
            } else {
              if self.direction == 1 {
                self.legs_apply_animation(state.anims.crouch_run.clone(), 1);
              } else {
                self.legs_apply_animation(state.anims.crouch_run_back.clone(), 1);
              }
            }

            if (self.legs_animation.id == state.anims.crouch_run.id)
              || (self.legs_animation.id == state.anims.crouch_run_back.id)
            {
              state.sprite_parts.forces[self.num].x = CROUCHRUNSPEED;
            } else if (self.legs_animation.id == state.anims.roll.id)
              || (self.legs_animation.id == state.anims.roll_back.id)
            {
              state.sprite_parts.forces[self.num].x = 2.0 * CROUCHRUNSPEED;
            }
          }
        // downleft
        } else if self.control.left && self.control.down {
          if self.on_ground {
            // roll to the side
            if (self.legs_animation.id == state.anims.run.id)
              || (self.legs_animation.id == state.anims.run_back.id)
              || (self.legs_animation.id == state.anims.fall.id)
              || (self.legs_animation.id == state.anims.prone_move.id)
              || ((self.legs_animation.id == state.anims.prone.id)
                && (self.legs_animation.curr_frame >= 24))
            {
              if (self.legs_animation.id == state.anims.prone_move.id)
                || ((self.legs_animation.id == state.anims.prone.id)
                  && (self.legs_animation.curr_frame == self.legs_animation.num_frames))
              {
                self.control.prone = false;
                self.position = POS_STAND;
              }

              if self.direction == 1 {
                self.body_apply_animation(state.anims.roll_back.clone(), 1);
                self.legs_animation = state.anims.roll_back.clone();
                self.legs_animation.curr_frame = 1;
              } else {
                self.body_apply_animation(state.anims.roll.clone(), 1);
                self.legs_animation = state.anims.roll.clone();
                self.legs_animation.curr_frame = 1;
              }
            } else {
              if self.direction == 1 {
                self.legs_apply_animation(state.anims.crouch_run_back.clone(), 1);
              } else {
                self.legs_apply_animation(state.anims.crouch_run.clone(), 1);
              }
            }

            if (self.legs_animation.id == state.anims.crouch_run.id)
              || (self.legs_animation.id == state.anims.crouch_run_back.id)
            {
              state.sprite_parts.forces[self.num].x = -CROUCHRUNSPEED;
            }
          }
        // Proning
        } else if (self.legs_animation.id == state.anims.prone.id)
          || (self.legs_animation.id == state.anims.prone_move.id)
          || ((self.legs_animation.id == state.anims.get_up.id)
            && (self.body_animation.id != state.anims.throw.id)
            && (self.body_animation.id != state.anims.punch.id))
        {
          if self.on_ground {
            if ((self.legs_animation.id == state.anims.prone.id)
              && (self.legs_animation.curr_frame > 25))
              || (self.legs_animation.id == state.anims.prone_move.id)
            {
              if self.control.left || self.control.right {
                if (self.legs_animation.curr_frame < 4) || (self.legs_animation.curr_frame > 14) {
                  state.sprite_parts.forces[self.num].x = {
                    if self.control.left {
                      -PRONESPEED
                    } else {
                      PRONESPEED
                    }
                  }
                }

                self.legs_apply_animation(state.anims.prone_move.clone(), 1);

                if (self.body_animation.id != state.anims.clip_in.id)
                  && (self.body_animation.id != state.anims.clip_out.id)
                  && (self.body_animation.id != state.anims.slide_back.id)
                  && (self.body_animation.id != state.anims.reload.id)
                  && (self.body_animation.id != state.anims.change.id)
                  && (self.body_animation.id != state.anims.throw.id)
                  && (self.body_animation.id != state.anims.throw_weapon.id)
                {
                  self.body_apply_animation(state.anims.prone_move.clone(), 1);
                }

                if self.legs_animation.id != state.anims.prone_move.id {
                  self.legs_animation = state.anims.prone_move.clone();
                }
              } else {
                if self.legs_animation.id != state.anims.prone.id {
                  self.legs_animation = state.anims.prone.clone();
                }
                self.legs_animation.curr_frame = 26;
              }
            }
          }
        } else if self.control.right && self.control.up {
          if self.on_ground {
            if (self.legs_animation.id == state.anims.run.id)
              || (self.legs_animation.id == state.anims.run_back.id)
              || (self.legs_animation.id == state.anims.stand.id)
              || (self.legs_animation.id == state.anims.crouch.id)
              || (self.legs_animation.id == state.anims.crouch_run.id)
              || (self.legs_animation.id == state.anims.crouch_run_back.id)
            {
              self.legs_apply_animation(state.anims.jump_side.clone(), 1);
            }

            if self.legs_animation.curr_frame == self.legs_animation.num_frames {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            }
          } else if (self.legs_animation.id == state.anims.roll.id)
            || (self.legs_animation.id == state.anims.roll_back.id)
          {
            if self.direction == 1 {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            } else {
              self.legs_apply_animation(state.anims.run_back.clone(), 1);
            }
          }
          if self.legs_animation.id == state.anims.jump.id {
            if self.legs_animation.curr_frame < 10 {
              self.legs_apply_animation(state.anims.jump_side.clone(), 1);
            }
          }

          if self.legs_animation.id == state.anims.jump_side.id {
            if (self.legs_animation.curr_frame > 3) && (self.legs_animation.curr_frame < 11) {
              state.sprite_parts.forces[self.num].x = JUMPDIRSPEED;
              state.sprite_parts.forces[self.num].y = -JUMPDIRSPEED / 1.2;
            }
          }
        } else if self.control.left && self.control.up {
          if self.on_ground {
            if (self.legs_animation.id == state.anims.run.id)
              || (self.legs_animation.id == state.anims.run_back.id)
              || (self.legs_animation.id == state.anims.stand.id)
              || (self.legs_animation.id == state.anims.crouch.id)
              || (self.legs_animation.id == state.anims.crouch_run.id)
              || (self.legs_animation.id == state.anims.crouch_run_back.id)
            {
              self.legs_apply_animation(state.anims.jump_side.clone(), 1);
            }

            if self.legs_animation.curr_frame == self.legs_animation.num_frames {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            }
          } else if (self.legs_animation.id == state.anims.roll.id)
            || (self.legs_animation.id == state.anims.roll_back.id)
          {
            if self.direction == -1 {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            } else {
              self.legs_apply_animation(state.anims.run_back.clone(), 1);
            }
          }

          if self.legs_animation.id == state.anims.jump.id {
            if self.legs_animation.curr_frame < 10 {
              self.legs_apply_animation(state.anims.jump_side.clone(), 1);
            }
          }

          if self.legs_animation.id == state.anims.jump_side.id {
            if (self.legs_animation.curr_frame > 3) && (self.legs_animation.curr_frame < 11) {
              state.sprite_parts.forces[self.num].x = -JUMPDIRSPEED;
              state.sprite_parts.forces[self.num].y = -JUMPDIRSPEED / 1.2;
            }
          }
        } else if self.control.up {
          if self.on_ground {
            if self.legs_animation.id != state.anims.jump.id {
              self.legs_apply_animation(state.anims.jump.clone(), 1);
            }
            if self.legs_animation.curr_frame == self.legs_animation.num_frames {
              self.legs_apply_animation(state.anims.stand.clone(), 1);
            }
          }
          if self.legs_animation.id == state.anims.jump.id {
            if (self.legs_animation.curr_frame > 8) && (self.legs_animation.curr_frame < 15) {
              state.sprite_parts.forces[self.num].y = -JUMPSPEED;
            }
            if self.legs_animation.curr_frame == self.legs_animation.num_frames {
              self.legs_apply_animation(state.anims.fall.clone(), 1);
            }
          }
        } else if self.control.down {
          if self.on_ground {
            self.legs_apply_animation(state.anims.crouch.clone(), 1);
          }
        } else if self.control.right {
          if true {
            // if self.para = 0
            if self.direction == 1 {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            } else {
              self.legs_apply_animation(state.anims.run_back.clone(), 1);
            }
          }

          if self.on_ground {
            state.sprite_parts.forces[self.num].x = RUNSPEED;
            state.sprite_parts.forces[self.num].y = -RUNSPEEDUP;
          } else {
            state.sprite_parts.forces[self.num].x = FLYSPEED;
          }
        } else if self.control.left {
          if true {
            // if self.para = 0
            if self.direction == -1 {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            } else {
              self.legs_apply_animation(state.anims.run_back.clone(), 1);
            }
          }

          if self.on_ground {
            state.sprite_parts.forces[self.num].x = -RUNSPEED;
            state.sprite_parts.forces[self.num].y = -RUNSPEEDUP;
          } else {
            state.sprite_parts.forces[self.num].x = -FLYSPEED;
          }
        } else {
          if self.on_ground {
            self.legs_apply_animation(state.anims.stand.clone(), 1);
          } else {
            self.legs_apply_animation(state.anims.fall.clone(), 1);
          }
        }
      }
      // Body animations

      if (self.legs_animation.id == state.anims.roll.id)
        && (self.body_animation.id != state.anims.roll.id)
      {
        self.body_apply_animation(state.anims.roll.clone(), 1)
      }
      if (self.body_animation.id == state.anims.roll.id)
        && (self.legs_animation.id != state.anims.roll.id)
      {
        self.legs_apply_animation(state.anims.roll.clone(), 1)
      }
      if (self.legs_animation.id == state.anims.roll_back.id)
        && (self.body_animation.id != state.anims.roll_back.id)
      {
        self.body_apply_animation(state.anims.roll_back.clone(), 1)
      }
      if (self.body_animation.id == state.anims.roll_back.id)
        && (self.legs_animation.id != state.anims.roll_back.id)
      {
        self.legs_apply_animation(state.anims.roll_back.clone(), 1)
      }

      if (self.body_animation.id == state.anims.roll.id)
        || (self.body_animation.id == state.anims.roll_back.id)
      {
        if self.legs_animation.curr_frame != self.body_animation.curr_frame {
          if self.legs_animation.curr_frame > self.body_animation.curr_frame {
            self.body_animation.curr_frame = self.legs_animation.curr_frame;
          } else {
            self.legs_animation.curr_frame = self.body_animation.curr_frame;
          }
        }
      }

      // Gracefully end a roll animation
      if ((self.body_animation.id == state.anims.roll.id)
        || (self.body_animation.id == state.anims.roll_back.id))
        && (self.body_animation.curr_frame == self.body_animation.num_frames)
      {
        // Was probably a roll
        if self.on_ground {
          if self.control.down {
            if self.control.left || self.control.right {
              if self.body_animation.id == state.anims.roll.id {
                self.legs_apply_animation(state.anims.crouch_run.clone(), 1);
              } else {
                self.legs_apply_animation(state.anims.crouch_run_back.clone(), 1);
              }
            } else {
              self.legs_apply_animation(state.anims.crouch.clone(), 15);
            }
          }
        // Was probably a backflip
        } else if (self.body_animation.id == state.anims.roll_back.id) && self.control.up {
          if self.control.left || self.control.right {
            // Run back or forward depending on facing direction and direction key pressed
            if (self.direction == 1) ^ (self.control.left) {
              self.legs_apply_animation(state.anims.run.clone(), 1);
            } else {
              self.legs_apply_animation(state.anims.run_back.clone(), 1);
            }
          } else {
            self.legs_apply_animation(state.anims.fall.clone(), 1);
          }
        // Was probably a roll (that ended mid-air)
        } else if self.control.down {
          if self.control.left || self.control.right {
            if self.body_animation.id == state.anims.roll.id {
              self.legs_apply_animation(state.anims.crouch_run.clone(), 1);
            } else {
              self.legs_apply_animation(state.anims.crouch_run_back.clone(), 1);
            }
          } else {
            self.legs_apply_animation(state.anims.crouch.clone(), 15);
          }
        }
        self.body_apply_animation(state.anims.stand.clone(), 1);
      }

      if (!self.control.grenade && (self.body_animation.id != state.anims.recoil.id)
        && (self.body_animation.id != state.anims.small_recoil.id)
        && (self.body_animation.id != state.anims.aim_recoil.id)
        && (self.body_animation.id != state.anims.hands_up_recoil.id)
        && (self.body_animation.id != state.anims.shotgun.id)
        && (self.body_animation.id != state.anims.barret.id)
        && (self.body_animation.id != state.anims.change.id)
        && (self.body_animation.id != state.anims.throw_weapon.id)
        && (self.body_animation.id != state.anims.weapon_none.id)
        && (self.body_animation.id != state.anims.punch.id)
        && (self.body_animation.id != state.anims.roll.id)
        && (self.body_animation.id != state.anims.roll_back.id)
        && (self.body_animation.id != state.anims.reload_bow.id)
        && (self.body_animation.id != state.anims.cigar.id)
        && (self.body_animation.id != state.anims.match_.id)
        && (self.body_animation.id != state.anims.smoke.id)
        && (self.body_animation.id != state.anims.wipe.id)
        && (self.body_animation.id != state.anims.take_off.id)
        && (self.body_animation.id != state.anims.groin.id)
        && (self.body_animation.id != state.anims.piss.id)
        && (self.body_animation.id != state.anims.mercy.id)
        && (self.body_animation.id != state.anims.mercy2.id)
        && (self.body_animation.id != state.anims.victory.id)
        && (self.body_animation.id != state.anims.own.id)
        && (self.body_animation.id != state.anims.reload.id)
        && (self.body_animation.id != state.anims.prone.id)
        && (self.body_animation.id != state.anims.get_up.id)
        && (self.body_animation.id != state.anims.prone_move.id)
        && (self.body_animation.id != state.anims.melee.id))
        || ((self.body_animation.curr_frame == self.body_animation.num_frames)
          && (self.body_animation.id != state.anims.prone.id))
      {
        if self.position != POS_PRONE {
          if self.position == POS_STAND {
            self.body_apply_animation(state.anims.stand.clone(), 1);
          }

          if self.position == POS_CROUCH {
            if self.collider_distance < 255 {
              if self.body_animation.id == state.anims.hands_up_recoil.id {
                self.body_apply_animation(state.anims.hands_up_aim.clone(), 11);
              } else {
                self.body_apply_animation(state.anims.hands_up_aim.clone(), 1);
              }
            } else {
              if self.body_animation.id == state.anims.aim_recoil.id {
                self.body_apply_animation(state.anims.aim.clone(), 6);
              } else {
                self.body_apply_animation(state.anims.aim.clone(), 1);
              }
            }
          }
        } else {
          self.body_apply_animation(state.anims.prone.clone(), 26);
        }
      }

      if (self.legs_animation.id == state.anims.crouch.id)
        || (self.legs_animation.id == state.anims.crouch_run.id)
        || (self.legs_animation.id == state.anims.crouch_run_back.id)
      {
        self.position = POS_CROUCH;
      } else {
        self.position = POS_STAND;
      }
      if (self.legs_animation.id == state.anims.prone.id)
        || (self.legs_animation.id == state.anims.prone_move.id)
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
