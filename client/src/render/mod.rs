use super::*;

pub mod bullets;
pub mod components;
pub mod debug;
pub mod game;
pub mod gfx;
pub mod map;
pub mod soldiers;
pub mod systems;

pub use self::game::{GameGraphics, Sprites};

use self::map::*;
use self::soldiers::*;
use gfx2d::*;
use std::{collections::VecDeque, path::PathBuf};

fn filename_override(fs: &Filesystem, prefix: &str, fname: &str) -> PathBuf {
    let path = PathBuf::from(fname);

    // Use / even if OS uses \, as gvfs supports / only.
    let mut path_segments = path
        .as_path()
        .iter()
        .map(|s| s.to_string_lossy())
        .filter(|s| !s.is_empty() && s != "/")
        .collect::<VecDeque<_>>();
    if !prefix.is_empty() {
        path_segments.push_front(prefix.into());
    }

    let mut path = PathBuf::from(format!("/{}", Vec::from(path_segments).join("/")));

    for ext in &["png", "jpg", "gif", "bmp"] {
        path.set_extension(ext);
        if fs.is_file(path.clone()) {
            break;
        }
    }

    path
}
