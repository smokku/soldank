use std::fs::File;
use na::Vector3;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const MAX_POS_INDEX: i32 = 20;
const MAX_FRAMES_INDEX: i32 = 40;

#[derive(Debug)]
pub struct Frames {
  pub pos: [Vector3<f32>; 20],
}

impl Copy for Frames {}

impl Clone for Frames {
  fn clone(&self) -> Frames {
    *self
  }
}

#[derive(Copy, Clone)]
pub struct Animation {
  pub id: i32,
  pub num_frames: i32,
  pub speed: i32,
  pub count: i32,
  pub curr_frame: i32,
  pub looped: bool,
  pub frame: [Frames; 40],
}
impl Animation {
  pub fn do_animation(&mut self) {
    self.count += 1;
    if self.count == self.speed {
      self.count = 0;
      self.curr_frame += 1;
      if self.curr_frame > self.num_frames {
        if self.looped {
          self.curr_frame = 1;
        } else {
          self.curr_frame = self.num_frames;
        }
      }
    }
  }

  pub fn load_from_file(file_name: &str, id: i32, speed: i32, looped: bool) -> Box<Animation> {
    let mut path = PathBuf::new();
    path.push("assets/anims/");
    path.push(file_name);
    let mut num_frames: i32 = 0;

    let file = File::open(&path).expect("Error opening File");

    let mut line = String::new();
    let mut buf = BufReader::new(file);

    buf.read_line(&mut line).ok();
    let pos: [Vector3<f32>; 20] = [Vector3::new(0.0_f32, 0.0_f32, 0.0_f32); 20];
    let mut new_frame: [Frames; 40] = [Frames { pos }; 40];

    while line.trim() != "ENDFILE" {
      if line.trim() == "NEXTFRAME" {
        if num_frames == MAX_FRAMES_INDEX {
          println!("Corrupted frame index: {}", path.display());
          break;
        }

        num_frames += 1;
      } else {
        let mut r2 = String::new();
        let mut r3 = String::new();
        let mut r4 = String::new();
        buf.read_line(&mut r2).ok();
        buf.read_line(&mut r3).ok();
        buf.read_line(&mut r4).ok();
        let p: i32 = line.trim().parse().unwrap_or(0);
        let p2: f32 = r2.trim().parse().unwrap_or(0.0);
        let p4: f32 = r4.trim().parse().unwrap_or(0.0);
        r2.clear();
        r3.clear();
        r4.clear();
        if (p >= 1) && (p <= MAX_POS_INDEX) {
          new_frame[num_frames as usize].pos[p as usize - 1].x = -3.0 * p2 / 1.1;
          new_frame[num_frames as usize].pos[p as usize - 1].y = -3.0 * p4;
        } else {
          println!("Corrupted Index ({}): {}", p, path.display());
        }
      }
      line.clear();
      let num_bytes = buf.read_line(&mut line);
      if num_bytes.unwrap() == 0 {
        break;
      }
    }

    Box::new(Animation {
      id: id,
      num_frames: num_frames,
      speed: speed,
      count: 0,
      curr_frame: 1,
      looped: looped,
      frame: new_frame,
    })
  }
}
