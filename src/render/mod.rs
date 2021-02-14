use super::*;

pub mod bullets;
pub mod game;
pub mod gfx;
pub mod map;
pub mod soldiers;

pub use self::game::GameGraphics;

use self::bullets::*;
use self::map::*;
use self::soldiers::*;
use gfx2d::*;
use std::path::PathBuf;

fn filename_override(fs: &Filesystem, prefix: &str, fname: &str) -> PathBuf {
    let mut path = PathBuf::from(prefix);
    path.push(fname);

    // Use / even if OS uses \, as gvfs supports / only.
    let path_string = path
        .as_path()
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    let mut path = PathBuf::from(path_string);

    for ext in &["png", "jpg", "gif", "bmp"] {
        path.set_extension(ext);
        if fs.is_file(path.clone()) {
            break;
        }
    }

    path
}
