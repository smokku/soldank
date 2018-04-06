use super::*;

pub mod game;
pub mod gfx;
pub mod gostek;
pub mod map;

pub use self::game::GameGraphics;

use self::gostek::*;
use self::map::*;
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
