use na::Vector2;

use shared::anims;
use shared::parts;
use shared::state::MainState;
use shared::anims::Animation;
use shared::parts::ParticleSystem;
use shared::calc;
use glutin;
use std::process;

const SLIDELIMIT: f32 = 0.2;
const GRAV: f32 = 0.06;
const SURFACECOEFX: f32 = 0.970;
const SURFACECOEFY: f32 = 0.970;
const CROUCHMOVESURFACECOEFX: f32 = 0.85;
const CROUCHMOVESURFACECOEFY: f32 = 0.97;
const STANDSURFACECOEFX: f32 = 0.00;
const STANDSURFACECOEFY: f32 = 0.00;

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
const MAX_VELOCITY: f32 = 11.0;

const DEFAULT_IDLETIME: i32 = SECOND * 8;

#[derive(Default, Debug)]
pub struct Control {
  left: bool,
  right: bool,
  up: bool,
  down: bool,
  fire: bool,
  jets: bool,
  grenade: bool,
  change: bool,
  throw: bool,
  reload: bool,
  prone: bool,
  flag_throw: bool,
  mouse_aim_x: i32,
  mouse_aim_y: i32,
  mouse_dist: i32,
  pub was_running_left: bool,
  pub was_jumping: bool,
  pub was_throwing_weapon: bool,
  pub was_changing_weapon: bool,
  pub was_throwing_grenade: bool,
  pub was_reloading_weapon: bool,
}
#[allow(dead_code)]
pub struct Sprite {
  active: bool,
  dead_meat: bool,
  style: u8,
  num: usize,
  visible: u8,
  on_ground: bool,
  on_ground_for_law: bool,
  on_ground_last_frame: bool,
  on_ground_permanent: bool,
  direction: i8,
  old_direction: i8,
  alpha: u8,
  jets_count: i32,
  jets_count_prev: i32,
  wear_helmet: u8,
  has_cigar: u8,
  idle_time: i32,
  idle_random: i8,
  position: u8,
  on_fire: u8,
  collider_distance: u8,
  half_dead: bool,
  skeleton: parts::ParticleSystem,
  legs_animation: Box<anims::Animation>,
  body_animation: Box<anims::Animation>,
  control: Control,
}

impl Sprite {
  pub fn update_keys(&mut self, input: &glutin::KeyboardInput) {
    match input.state {
      glutin::ElementState::Pressed => match input.virtual_keycode {
        Some(glutin::VirtualKeyCode::A) => self.control.left = true,
        Some(glutin::VirtualKeyCode::D) => self.control.right = true,
        Some(glutin::VirtualKeyCode::W) => self.control.up = true,
        Some(glutin::VirtualKeyCode::S) => self.control.down = true,
        Some(glutin::VirtualKeyCode::Q) => self.control.change = true,
        Some(glutin::VirtualKeyCode::E) => self.control.throw = true,
        Some(glutin::VirtualKeyCode::X) => self.control.prone = true,
        Some(glutin::VirtualKeyCode::Escape) => process::exit(0x0100),
        _ => {}
      },
      glutin::ElementState::Released => match input.virtual_keycode {
        Some(glutin::VirtualKeyCode::A) => self.control.left = false,
        Some(glutin::VirtualKeyCode::D) => self.control.right = false,
        Some(glutin::VirtualKeyCode::W) => self.control.up = false,
        Some(glutin::VirtualKeyCode::S) => self.control.down = false,
        Some(glutin::VirtualKeyCode::Q) => self.control.change = false,
        Some(glutin::VirtualKeyCode::E) => self.control.throw = false,
        Some(glutin::VirtualKeyCode::X) => self.control.prone = false,
        _ => {}
      },
    }
  }

  pub fn update_mouse_button(&mut self, input: &(glutin::ElementState, glutin::MouseButton)) {
    let pressed = match input.0 {
      glutin::ElementState::Pressed => true,
      glutin::ElementState::Released => false,
    };
    match input.1 {
      glutin::MouseButton::Left => self.control.fire = pressed,
      glutin::MouseButton::Right => self.control.jets = pressed,
      _ => (),
    }
  }

  pub fn new(state: &mut MainState) -> Sprite {
    let control: Control = Default::default();
    let mut gostek = ParticleSystem::new();
    gostek.load_from_file(&String::from("gostek.po"), 4.50);
    gostek.timestep = 1.00;
    gostek.gravity = 1.06 * GRAV;
    gostek.v_damping = 0.9945;
    state.sprite_parts.create_part(
      Vector2::new(state.map.spawnpoints[0].x as f32, state.map.spawnpoints[0].y as f32),
      Vector2::new(0.0f32, 0.0f32),
      1.00,
      1,
    );
    Sprite {
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
      alpha: 255,
      jets_count: 0,
      jets_count_prev: 0,
      wear_helmet: 1,
      has_cigar: 1,
      idle_time: 0,
      idle_random: 0,
      position: 0,
      on_fire: 0,
      collider_distance: 255,
      half_dead: false,
      skeleton: gostek,
      legs_animation: state.anims.stand.clone(),
      body_animation: state.anims.stand.clone(),
      control: control,
    }
  }
  pub fn legs_apply_animation(&mut self, anim: Box<Animation>, curr: i32) {
    /*
    if (LegsAnimation.ID = Prone.ID) or
     (LegsAnimation.ID = ProneMove.ID) then
    */
    if (self.legs_animation.id == 34) || (self.legs_animation.id == 38) {
      return;
    }
    if anim.id != self.legs_animation.id {
      self.legs_animation = anim;
      self.legs_animation.curr_frame = curr;
    }
  }
  pub fn body_apply_animation(&mut self, anim: Box<Animation>, curr: i32) {
    if anim.id != self.body_animation.id {
      self.body_animation = anim;
      self.body_animation.curr_frame = curr;
    }
  }

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
      self.free_controls();
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

  pub fn update(&mut self, state: &mut MainState) {
    let mut body_y = 0.0;

    let mut arm_s;

    self.control(state);

    self.skeleton.old_pos[21] = self.skeleton.pos[21];
    self.skeleton.old_pos[23] = self.skeleton.pos[23];
    self.skeleton.old_pos[25] = self.skeleton.pos[25];
    self.skeleton.pos[21] = self.skeleton.pos[9];
    self.skeleton.pos[23] = self.skeleton.pos[12];
    self.skeleton.pos[25] = self.skeleton.pos[5];

    if !self.dead_meat {
      self.skeleton.pos[21] += state.sprite_parts.velocity[self.num];
      self.skeleton.pos[23] += state.sprite_parts.velocity[self.num];
      self.skeleton.pos[25] += state.sprite_parts.velocity[self.num];
    }

    match self.position {
      POS_STAND => body_y = 8.0,
      POS_CROUCH => body_y = 9.0,
      POS_PRONE => {
        if self.body_animation.id == state.anims.prone.id {
          if self.body_animation.curr_frame > 9 {
            body_y = -2.0
          } else {
            body_y = 14.0 - self.body_animation.curr_frame as f32;
          }
        } else {
          body_y = 9.0;
        }

        if self.body_animation.id == state.anims.prone_move.id {
          body_y = 0.0;
        }
      }
      _ => {}
    }

    if self.body_animation.id == state.anims.get_up.id {
      if self.body_animation.curr_frame > 18 {
        body_y = 8.0;
      } else {
        body_y = 4.0;
      }
    }

    if self.control.mouse_aim_x as f32 >= state.sprite_parts.pos[self.num].x {
      self.direction = 1;
    } else {
      self.direction = -1;
    }

    for i in 1..20 {
      if self.skeleton.active[i] && !self.dead_meat {
        self.skeleton.old_pos[i] = self.skeleton.pos[i];

        if !self.half_dead {
          if (i == 1) || (i == 4) || (i == 2) || (i == 3) || (i == 5) || (i == 6) || (i == 17)
            || (i == 18)
          {
            // legs
            self.skeleton.pos[i].x = state.sprite_parts.pos[i].x
              + f32::from(self.direction)
                * self.legs_animation.frame[self.legs_animation.curr_frame as usize].pos[i].x;
            self.skeleton.pos[i].y = state.sprite_parts.pos[i].y
              + f32::from(self.direction)
                * self.legs_animation.frame[self.legs_animation.curr_frame as usize].pos[i].y;
          }
        }
        if (i == 7) || (i == 8) || (i == 9) || (i == 10) || (i == 11) || (i == 12) || (i == 13)
          || (i == 14) || (i == 15) || (i == 16) || (i == 19) || (i == 20)
        {
          self.skeleton.pos[i].x = state.sprite_parts.pos[i].x
            + f32::from(self.direction)
              * self.body_animation.frame[self.body_animation.curr_frame as usize].pos[i].x;

          if !self.half_dead {
            self.skeleton.pos[i].y = (self.skeleton.pos[6].y
              - (state.sprite_parts.pos[self.num].y - body_y))
              + state.sprite_parts.pos[self.num].y
              + self.body_animation.frame[self.body_animation.curr_frame as usize].pos[i].y;
          } else {
            self.skeleton.pos[i].y = 9.00 + state.sprite_parts.pos[self.num].y
              + self.body_animation.frame[self.body_animation.curr_frame as usize].pos[i].y;
          }
        }
      }
    }

    let mut i = 12;

    if !self.dead_meat {
      let p = Vector2::new(self.skeleton.pos[i].x, self.skeleton.pos[i].y);

      let mouse_aim = Vector2::new(
        self.control.mouse_aim_x as f32,
        self.control.mouse_aim_y as f32,
      );
      let mut r_norm = p - mouse_aim;
      r_norm = calc::vec2normalize(r_norm, r_norm);
      r_norm *= 0.1;
      self.skeleton.pos[i].x = self.skeleton.pos[9].x - f32::from(self.direction) * r_norm.y;
      self.skeleton.pos[i].y = self.skeleton.pos[9].y + f32::from(self.direction) * r_norm.x;

      r_norm *= 50.0;

      self.skeleton.pos[23].x = self.skeleton.pos[9].x - f32::from(self.direction) * r_norm.y;
      self.skeleton.pos[23].y = self.skeleton.pos[9].y + f32::from(self.direction) * r_norm.x;
    }

    if self.body_animation.id == state.anims.throw.id {
      arm_s = -5.00;
    } else {
      arm_s = -7.00;
    }

    i = 15;

    if (self.body_animation.id != state.anims.reload.id)
      && (self.body_animation.id != state.anims.reload_bow.id)
      && (self.body_animation.id != state.anims.clip_in.id)
      && (self.body_animation.id != state.anims.clip_out.id)
      && (self.body_animation.id != state.anims.slide_back.id)
      && (self.body_animation.id != state.anims.change.id)
      && (self.body_animation.id != state.anims.throw_weapon.id)
      && (self.body_animation.id != state.anims.punch.id)
      && (self.body_animation.id != state.anims.roll.id)
      && (self.body_animation.id != state.anims.roll_back.id)
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
      && (self.body_animation.id != state.anims.melee.id)
    {
      let p = Vector2::new(self.skeleton.pos[i].x, self.skeleton.pos[i].y);
      let mouse_aim = Vector2::new(
        self.control.mouse_aim_x as f32,
        self.control.mouse_aim_y as f32,
      );
      let mut r_norm = p - mouse_aim;
      r_norm = calc::vec2normalize(r_norm, r_norm);
      r_norm *= arm_s;
      let m = Vector2::new(self.skeleton.pos[16].x, self.skeleton.pos[16].y);
      let p = m + r_norm;
      self.skeleton.pos[i].x = p.x;
      self.skeleton.pos[i].y = p.y;
    }

    if self.body_animation.id == state.anims.throw.id {
      arm_s = -6.00;
    } else {
      arm_s = -8.00;
    }

    i = 19;

    if (self.body_animation.id != state.anims.reload.id)
      && (self.body_animation.id != state.anims.reload_bow.id)
      && (self.body_animation.id != state.anims.clip_in.id)
      && (self.body_animation.id != state.anims.clip_out.id)
      && (self.body_animation.id != state.anims.slide_back.id)
      && (self.body_animation.id != state.anims.change.id)
      && (self.body_animation.id != state.anims.throw_weapon.id)
      && (self.body_animation.id != state.anims.punch.id)
      && (self.body_animation.id != state.anims.roll.id)
      && (self.body_animation.id != state.anims.roll_back.id)
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
      && (self.body_animation.id != state.anims.melee.id)
    {
      let p = Vector2::new(self.skeleton.pos[i].x, self.skeleton.pos[i].y);
      let mouse_aim = Vector2::new(
        self.control.mouse_aim_x as f32,
        self.control.mouse_aim_y as f32,
      );
      let mut r_norm = p - mouse_aim;
      r_norm = calc::vec2normalize(r_norm, r_norm);
      r_norm *= arm_s;
      let m = Vector2::new(self.skeleton.pos[16].x, self.skeleton.pos[16].y - 4.0);
      let p = m + r_norm;
      self.skeleton.pos[i].x = p.x;
      self.skeleton.pos[i].y = p.y;
    }

    if !self.dead_meat {
      self.body_animation.do_animation();
      self.legs_animation.do_animation();

      self.on_ground = false;

      let position = Vector2::new(
        state.sprite_parts.pos[self.num].x,
        state.sprite_parts.pos[self.num].y,
      );

      self.check_map_collision(state, position.x - 3.5, position.y - 12.0.clone(), 1);
      let mut position = Vector2::new(
        state.sprite_parts.pos[self.num].x,
        state.sprite_parts.pos[self.num].y,
      );
      self.check_map_collision(state, position.x + 3.5, position.y - 12.0.clone(), 1);

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
        //let leg_vector = Vector2::new(
        //  state.sprite_parts.pos[self.num].x + 2.0,
        //  state.sprite_parts.pos[self.num].y + 1.9,
        //);
        //    if Map.RayCast(LegVector, LegVector, LegDistance, 10) {
        body_y = 0.25;
        // }
      }
      if arm_s == 0.0 {
        //let leg_vector = Vector2::new(
        //  state.sprite_parts.pos[self.num].x - 2.0,
        //  state.sprite_parts.pos[self.num].y + 1.9,
        //);
        //    if Map.RayCast(LegVector, LegVector, LegDistance, 10) {
        arm_s = 0.25;
        // }
      }
      position = Vector2::new(
        state.sprite_parts.pos[self.num].x,
        state.sprite_parts.pos[self.num].y,
      );
      self.on_ground =
        self.check_map_collision(state, position.x + 2.0, position.y + 2.0 - body_y, 0);
      position = Vector2::new(
        state.sprite_parts.pos[self.num].x,
        state.sprite_parts.pos[self.num].y,
      );
      self.on_ground = self.on_ground
        || self.check_map_collision(state, position.x - 2.0, position.y + 2.0 - arm_s, 0);
      position = Vector2::new(
        state.sprite_parts.pos[self.num].x,
        state.sprite_parts.pos[self.num].y,
      );
      let grounded = self.on_ground;
      self.on_ground =
        self.check_map_vertices_collision(state, position.x, position.y, 3.00, grounded)
          || self.on_ground;
      //    OnGround or OnGroundForLaw) or OnGround;
      if !(self.on_ground ^ self.on_ground_last_frame) {
        self.on_ground_permanent = self.on_ground;
      }

      self.on_ground_last_frame = self.on_ground;

      if (self.jets_count < state.map.start_jet) && !(self.control.jets) {
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

      state.sprite_parts.pos[self.num] = self.skeleton.pos[12];

      //CheckSkeletonOutOfBounds;
    }

    if state.sprite_parts.velocity[self.num].x > MAX_VELOCITY {
      state.sprite_parts.velocity[self.num].x = MAX_VELOCITY;
    }
    if state.sprite_parts.velocity[self.num].x < -MAX_VELOCITY {
      state.sprite_parts.velocity[self.num].x = -MAX_VELOCITY;
    }
    if state.sprite_parts.velocity[self.num].y > MAX_VELOCITY {
      state.sprite_parts.velocity[self.num].y = MAX_VELOCITY;
    }
    if state.sprite_parts.velocity[self.num].y < -MAX_VELOCITY {
      state.sprite_parts.velocity[self.num].y = MAX_VELOCITY;
    }
  }
  pub fn check_map_collision(&mut self, state: &mut MainState, x: f32, y: f32, area: i32) -> bool {
    let s_pos = Vector2::new(x, y);

    let pos = Vector2::new(
      s_pos.x + state.sprite_parts.velocity[self.num].x,
      s_pos.y + state.sprite_parts.velocity[self.num].y,
    );
    let rx = ((pos.x / state.map.sectors_division as f32).round()) as i32 + 25;
    let ry = ((pos.y / state.map.sectors_division as f32).round()) as i32 + 25;

    if (rx > 0) && (rx < state.map.sectors_num + 25) && (ry > 0)
      && (ry < state.map.sectors_num + 25)
    {
      for j in 0..state.map.sectors_poly[rx as usize][ry as usize].polys.len() {
        let w = state.map.sectors_poly[rx as usize][ry as usize].polys[j] - 1;

        let mut polygons = state.map.polygons[w as usize];
        if state.map.point_in_poly(pos, &mut polygons) {
          let mut d = 0.0;

          let mut k = 0;

          let mut perp = state
            .map
            .closest_perpendicular(w as i32, pos, &mut d, &mut k);

          let step = perp;

          perp = calc::vec2normalize(perp, perp);

          perp *= d;

          d = calc::vec2length(state.sprite_parts.velocity[self.num]);

          if calc::vec2length(perp) > d {
            perp = calc::vec2normalize(perp, perp);
            perp *= d;
          }
          if (area == 0)
            || ((area == 1)
              && ((state.sprite_parts.velocity[self.num].y < 0.0)
                || (state.sprite_parts.velocity[self.num].x > SLIDELIMIT)
                || (state.sprite_parts.velocity[self.num].x < -SLIDELIMIT)))
          {
            state.sprite_parts.old_pos[self.num] = state.sprite_parts.pos[self.num];
            state.sprite_parts.pos[self.num] -= perp;
            state.sprite_parts.velocity[self.num] -= perp;
          }

          if area == 0 {
            if (self.legs_animation.id == state.anims.stand.id)
              || (self.legs_animation.id == state.anims.crouch.id)
              || (self.legs_animation.id == state.anims.prone.id)
              || (self.legs_animation.id == state.anims.prone_move.id)
              || (self.legs_animation.id == state.anims.get_up.id)
              || (self.legs_animation.id == state.anims.fall.id)
              || (self.legs_animation.id == state.anims.mercy.id)
              || (self.legs_animation.id == state.anims.mercy2.id)
              || (self.legs_animation.id == state.anims.own.id)
            {
              if (state.sprite_parts.velocity[self.num].x < SLIDELIMIT)
                && (state.sprite_parts.velocity[self.num].x > -SLIDELIMIT)
                && (step.y > SLIDELIMIT)
              {
                state.sprite_parts.pos[self.num] = state.sprite_parts.old_pos[self.num];
                state.sprite_parts.forces[self.num].y -= GRAV;
              }

              /* (PolyType <> POLY_TYPE_ICE) and (PolyType <> POLY_TYPE_BOUNCY) */
              if step.y > SLIDELIMIT {
                if (self.legs_animation.id == state.anims.stand.id)
                  || (self.legs_animation.id == state.anims.fall.id)
                  || (self.legs_animation.id == state.anims.crouch.id)
                {
                  state.sprite_parts.velocity[self.num].x *= STANDSURFACECOEFX;
                  state.sprite_parts.velocity[self.num].y *= STANDSURFACECOEFY;

                  state.sprite_parts.forces[self.num].x -= state.sprite_parts.velocity[self.num].x;
                } else if self.legs_animation.id == state.anims.prone.id {
                  if self.legs_animation.curr_frame > 24 {
                    if !(self.control.down && (self.control.left || self.control.right)) {
                      state.sprite_parts.velocity[self.num].x *= STANDSURFACECOEFX;
                      state.sprite_parts.velocity[self.num].y *= STANDSURFACECOEFY;

                      state.sprite_parts.forces[self.num].x -=
                        state.sprite_parts.velocity[self.num].x;
                    }
                  } else {
                    state.sprite_parts.velocity[self.num].x *= SURFACECOEFX;
                    state.sprite_parts.velocity[self.num].y *= SURFACECOEFY;
                  }
                } else if self.legs_animation.id == state.anims.get_up.id {
                  state.sprite_parts.velocity[self.num].x *= SURFACECOEFX;
                  state.sprite_parts.velocity[self.num].y *= SURFACECOEFY;
                } else if self.legs_animation.id == state.anims.prone_move.id {
                  state.sprite_parts.velocity[self.num].x *= STANDSURFACECOEFX;
                  state.sprite_parts.velocity[self.num].y *= STANDSURFACECOEFY;
                }
              }
            } else {
              if (self.legs_animation.id == state.anims.crouch_run.id)
                || (self.legs_animation.id == state.anims.crouch_run_back.id)
              {
                state.sprite_parts.velocity[self.num].x *= CROUCHMOVESURFACECOEFX;
                state.sprite_parts.velocity[self.num].y *= CROUCHMOVESURFACECOEFY;
              } else {
                state.sprite_parts.velocity[self.num].x *= SURFACECOEFX;
                state.sprite_parts.velocity[self.num].y *= SURFACECOEFY;
              }
            }
          }
          return true;
        }
      }
    }
    false
  }

  pub fn check_map_vertices_collision(
    &mut self,
    state: &mut MainState,
    x: f32,
    y: f32,
    r: f32,
    has_collided: bool,
  ) -> bool {
    let s_pos = Vector2::new(x, y);

    let pos = Vector2::new(
      s_pos.x + state.sprite_parts.velocity[self.num].x,
      s_pos.y + state.sprite_parts.velocity[self.num].y,
    );
    let rx = ((pos.x / state.map.sectors_division as f32).round()) as i32 + 25;
    let ry = ((pos.y / state.map.sectors_division as f32).round()) as i32 + 25;

    if (rx > 0) && (rx < state.map.sectors_num + 25) && (ry > 0)
      && (ry < state.map.sectors_num + 25)
    {
      for j in 0..state.map.sectors_poly[rx as usize][ry as usize].polys.len() {
        let w = state.map.sectors_poly[rx as usize][ry as usize].polys[j] - 1;

        for i in 0..3 {
          let vert = Vector2::new(
            state.map.polygons[w as usize].vertices[i].x,
            state.map.polygons[w as usize].vertices[i].y,
          );

          if !has_collided {
            // handle_special_polytypes(polytype, pos);
          }

          let d = calc::distance(vert, pos);
          if d < r {
            let mut dir = pos - vert;
            dir = calc::vec2normalize(dir, dir);
            state.sprite_parts.pos[self.num] += dir;
            return true;
          }
        }
      }
    }
    // for i in 1..
    //}
    false
  }
  pub fn free_controls(&mut self) {
    self.control = Default::default();
  }
}
