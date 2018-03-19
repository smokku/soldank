mod map;
mod game;
mod sprite_data;

pub use self::map::*;
pub use self::game::*;
pub use self::sprite_data::*;

fn filename_override(prefix: &str, fname: &str) -> ::std::path::PathBuf {
    let mut path = ::std::path::PathBuf::from(prefix);
    path.push(fname);

    for ext in &["png", "jpg", "gif", "bmp"] {
        path.set_extension(ext);
        if path.exists() { break; }
    }

    path
}
