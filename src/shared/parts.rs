use super::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const NUM_PARTICLES: i32 = 560;

#[derive(Debug, Copy, Clone)]
pub struct Constraint {
  pub active: bool,
  pub part_a: i32,
  pub part_b: i32,
  pub rest_length: f32,
}

pub struct ParticleSystem {
  pub active: [bool; 560],
  pub pos: [Vector2<f32>; 560],
  pub velocity: [Vector2<f32>; 560],
  pub old_pos: [Vector2<f32>; 560],
  pub forces: [Vector2<f32>; 560],
  pub one_over_mass: [f32; 560],
  pub timestep: f32,
  pub gravity: f32,
  pub v_damping: f32,
  pub e_damping: f32,
  pub constraint_count: i32,
  pub part_count: i32,
  pub constraints: [Constraint; 560],
}

impl ParticleSystem {
  pub fn new() -> ParticleSystem {
    let active: [bool; 560] = [false; 560];
    let pos: [Vector2<f32>; 560] = [Vector2::zeros(); 560];
    let velocity: [Vector2<f32>; 560] = [Vector2::zeros(); 560];
    let old_pos: [Vector2<f32>; 560] = [Vector2::zeros(); 560];
    let forces: [Vector2<f32>; 560] = [Vector2::zeros(); 560];
    let one_over_mass: [f32; 560] = [0.00f32; 560];
    let constraints: [Constraint; 560] = [Constraint {
      active: false,
      part_a: 0,
      part_b: 0,
      rest_length: 0.00f32,
    }; 560];
    ParticleSystem {
      active,
      pos,
      velocity,
      old_pos,
      forces,
      one_over_mass,
      timestep: 0.00f32,
      gravity: 0.00f32,
      v_damping: 0.00f32,
      e_damping: 0.00f32,
      constraint_count: 0,
      part_count: 0,
      constraints,
    }
  }

  pub fn do_verlet_timestep(&mut self) {
    for i in 1..NUM_PARTICLES {
      if self.active[i as usize] {
        self.verlet(i);
      }
    }
    self.satisfy_contstraints();
  }

  pub fn do_verlet_timestep_for(&mut self, i: i32, j: i32) {
    self.verlet(i);
    self.satisfy_contstraints_for(j);
  }

  #[allow(dead_code)]
  pub fn do_eurler_timestep(&mut self) {
    for i in 1..NUM_PARTICLES {
      if self.active[i as usize] {
        self.euler(i);
      }
    }
  }

  pub fn do_eurler_timestep_for(&mut self, i: i32) {
    self.euler(i)
  }

  pub fn euler(&mut self, i: i32) {
    // Accumulate Forces
    let temp_pos = self.pos[i as usize];
    self.forces[i as usize].y += self.gravity;

    let mut s: Vector2<f32> = self.forces[i as usize] * self.one_over_mass[i as usize];
    s *= self.timestep.powi(2);

    self.velocity[i as usize] += s;
    self.pos[i as usize] += self.velocity[i as usize];
    self.velocity[i as usize] *= self.e_damping;
    self.old_pos[i as usize] = temp_pos;

    self.forces[i as usize].x = 0.0f32;
    self.forces[i as usize].y = 0.0f32;
  }

  pub fn verlet(&mut self, i: i32) {
    // Accumulate Forces
    let temp_pos = Vector2::new(0.0f32, 0.0f32);
    self.forces[i as usize].y += self.gravity;

    let mut s1: Vector2<f32> = self.pos[i as usize] * (1.00 + self.v_damping);
    let mut s2: Vector2<f32> = self.old_pos[i as usize] * self.v_damping;

    let d = s1 - s2;

    s1 = self.forces[i as usize] * self.one_over_mass[i as usize];
    s2 = s1 * self.timestep.exp2();

    self.pos[i as usize] = d + s2;
    self.old_pos[i as usize] = temp_pos;

    self.forces[i as usize].x = 0.0f32;
    self.forces[i as usize].y = 0.0f32;
  }

  pub fn satisfy_contstraints(&mut self) {
    if self.constraint_count > 0 {
      for i in 1..self.constraint_count + 1 {
        if self.constraints[i as usize].active {
          let mut diff = 0.0;
          let delta = Vector2::new(self.constraints[i as usize].part_b as f32, 0.0f32)
            - Vector2::new(self.constraints[i as usize].part_b as f32, 0.0f32);
          let delta_length: f32 = (delta.x * delta.x + delta.x * delta.y).sqrt();
          if delta_length != 0.0 {
            diff = (delta_length - self.constraints[i as usize].rest_length) / delta_length;
          }
          if self.one_over_mass[self.constraints[i as usize].part_a as usize] > 0.0 {
            let d = delta * (0.5 * diff);
            self.pos[self.constraints[i as usize].part_a as usize] += d;
          }
          if self.one_over_mass[self.constraints[i as usize].part_b as usize] > 0.0 {
            let d = delta * (0.5 * diff);
            self.pos[self.constraints[i as usize].part_a as usize] += d;
          }
        }
      }
    }
  }

  pub fn satisfy_contstraints_for(&mut self, i: i32) {
    let mut diff = 0.0;
    let delta = Vector2::new(self.constraints[i as usize].part_b as f32, 0.0f32)
      - Vector2::new(self.constraints[i as usize].part_b as f32, 0.0f32);
    let delta_length: f32 = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if delta_length != 0.0 {
      diff = (delta_length - self.constraints[i as usize].rest_length) / delta_length;
    }
    if self.one_over_mass[self.constraints[i as usize].part_a as usize] > 0.0 {
      let d = delta * (0.5 * diff);
      self.pos[self.constraints[i as usize].part_a as usize] += d;
    }
    if self.one_over_mass[self.constraints[i as usize].part_b as usize] > 0.0 {
      let d = delta * (0.5 * diff);
      self.pos[self.constraints[i as usize].part_a as usize] += d;
    }
  }

  pub fn create_part(&mut self, start: Vector2<f32>, vel: Vector2<f32>, mass: f32, num: i32) {
    self.active[num as usize] = true;
    self.pos[num as usize] = start;
    self.velocity[num as usize] = vel;
    self.old_pos[num as usize] = start;
    self.one_over_mass[num as usize] = 1.00 / mass;
  }

  pub fn make_constraint(&mut self, pa: i32, pb: i32, rest: f32) {
    self.constraint_count += 1;
    self.constraints[self.constraint_count as usize].active = true;
    self.constraints[self.constraint_count as usize].part_a = pa;
    self.constraints[self.constraint_count as usize].part_b = pb;
    self.constraints[self.constraint_count as usize].rest_length = rest;
  }

  pub fn load_from_file(&mut self, file_name: &str, scale: f32) {
    let mut path = PathBuf::new();
    path.push("assets/objects/");
    path.push(file_name);

    let mut i: i32 = 0;
    let file = File::open(&path).expect("Error opening File");
    let mut line = String::new();
    let mut buf = BufReader::new(file);

    let read_line = |buf: &mut BufReader<File>, line: &mut String| {
      line.clear();
      buf.read_line(line).ok();
    };

    let read_f32 = |buf: &mut BufReader<File>, line: &mut String| -> f32 {
      read_line(buf, line);
      line.trim().parse().unwrap()
    };

    let read_i32 = |line: &str| -> i32 {
      let mut chars = line.chars();
      chars.next();
      chars.as_str().trim().parse().unwrap()
    };

    read_line(&mut buf, &mut line);

    while line.trim() != "CONSTRAINTS" {
      let x = read_f32(&mut buf, &mut line);
      let _ = read_f32(&mut buf, &mut line);
      let z = read_f32(&mut buf, &mut line);

      i += 1;
      self.create_part(vec2(-x * scale / 1.2, -z * scale), vec2(0.0, 0.0), 1.0, i);
      read_line(&mut buf, &mut line);
    }

    self.part_count = i;

    loop {
      let pa = {
        read_line(&mut buf, &mut line);
        if line.is_empty() || line.trim() == "ENDFILE" {
          break;
        }
        read_i32(&line)
      };

      let pb = {
        read_line(&mut buf, &mut line);
        read_i32(&line)
      };

      let delta = self.pos[pa as usize] - self.pos[pb as usize];
      self.make_constraint(pa, pb, (delta.x * delta.x + delta.y * delta.y).sqrt());
    }
  }
}
