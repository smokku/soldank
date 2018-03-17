#![crate_type = "lib"]
#![crate_name = "gfx2d"]

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

include!("gfx_types.rs");

mod context;
mod texture;
mod batch;
mod transform;
mod spritesheet;
mod vec2;
mod matrix;
mod color;

pub mod binpack;
pub use context::Gfx2dContext;
pub use context::vertex;
pub use context::Vertex;
pub use texture::Texture;
pub use batch::DrawBatch;
pub use batch::DrawSlice;
pub use spritesheet::Sprite;
pub use spritesheet::SpriteInfo;
pub use spritesheet::Spritesheet;
pub use vec2::vec2;
pub use vec2::Vec2;
pub use matrix::Mat2d;
pub use transform::Transform;
pub use color::Color;
pub use color::rgb;
pub use color::rgba;
pub use gfx::texture::FilterMethod;
pub use gfx::texture::WrapMode;

pub mod gfx2d_extra {
    pub use super::texture::premultiply_image;
    pub use super::texture::remove_color_key;
}
