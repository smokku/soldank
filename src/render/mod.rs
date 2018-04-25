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

fn filename_override(prefix: &str, fname: &str) -> ::std::path::PathBuf {
    let mut path = ::std::path::PathBuf::from(prefix);
    path.push(fname);

    for ext in &["png", "jpg", "gif", "bmp"] {
        path.set_extension(ext);
        if path.exists() {
            break;
        }
    }

    path
}
