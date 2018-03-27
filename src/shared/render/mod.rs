use super::*;

pub mod map;
pub mod game;
pub mod gostek;
pub mod sprite_data;

pub use self::game::GameGraphics;

use gfx2d::*;
use self::sprite_data::*;
use self::gostek::*;
use self::map::*;

fn filename_override(prefix: &str, fname: &str) -> ::std::path::PathBuf {
    let mut path = ::std::path::PathBuf::from(prefix);
    path.push(fname);

    for ext in &["png", "jpg", "gif", "bmp"] {
        path.set_extension(ext);
        if path.exists() { break; }
    }

    path
}
