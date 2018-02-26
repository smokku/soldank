use na::Vector2;

use shared::anims;
use shared::parts;
use shared::state::MainState;
use shared::anims::Animation;
use shared::parts::ParticleSystem;
use shared::calc;
use shared::control::Control;
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

const MAX_VELOCITY: f32 = 11.0;

#[allow(dead_code)]
pub struct Sprite {
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
  pub alpha: u8,
  pub jets_count: i32,
  pub jets_count_prev: i32,
  pub wear_helmet: u8,
  pub has_cigar: u8,
  pub idle_time: i32,
  pub idle_random: i8,
  pub position: u8,
  pub on_fire: u8,
  pub collider_distance: u8,
  pub half_dead: bool,
  pub skeleton: parts::ParticleSystem,
  pub legs_animation: Box<anims::Animation>,
  pub body_animation: Box<anims::Animation>,
  pub control: Control,
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
      Vector2::new(
        state.map.spawnpoints[0].x as f32,
        state.map.spawnpoints[0].y as f32,
      ),
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

      self.check_map_collision(state, position.x - 3.5, position.y - 12.0, 1);
      let mut position = Vector2::new(
        state.sprite_parts.pos[self.num].x,
        state.sprite_parts.pos[self.num].y,
      );
      self.check_map_collision(state, position.x + 3.5, position.y - 12.0, 1);

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
}
