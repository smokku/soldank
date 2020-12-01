#![crate_type = "lib"]
#![crate_name = "gfx2d"]

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glam;
extern crate glutin;
extern crate image;

include!("gfx_types.rs");

mod batch;
mod color;
mod context;
mod spritesheet;
mod texture;
mod transform;

pub mod binpack;
pub mod math;

pub use batch::DrawBatch;
pub use batch::DrawSlice;
pub use color::rgb;
pub use color::rgba;
pub use color::Color;
pub use context::vertex;
pub use context::Gfx2dContext;
pub use context::Vertex;
pub use gfx::texture::FilterMethod;
pub use gfx::texture::WrapMode;
pub use spritesheet::Sprite;
pub use spritesheet::SpriteInfo;
pub use spritesheet::Spritesheet;
pub use texture::Texture;
pub use transform::Transform;

pub mod gfx2d_extra {
    pub use super::texture::load_image_rgba;
    pub use super::texture::premultiply_image;
    pub use super::texture::remove_color_key;
}

use math::*;
