pub use image;
pub use macroquad;

mod batch;
mod color;
mod extra;
mod spritesheet;
mod transform;

pub mod binpack;
pub mod math;

pub use batch::vertex;
pub use batch::DrawBatch;
pub use batch::DrawSlice;
pub use color::rgb;
pub use color::rgba;
pub use color::Color;
pub use macroquad::miniquad::{
    Buffer as VertexBuffer, FilterMode, Texture, TextureAccess, TextureFormat, TextureParams,
    TextureWrap,
};
pub use macroquad::prelude::Texture2D;
pub use spritesheet::Sprite;
pub use spritesheet::SpriteInfo;
pub use spritesheet::Spritesheet;
pub use transform::Transform;

pub mod gfx2d_extra {
    pub use super::extra::load_image_rgba;
    pub use super::extra::premultiply_image;
    pub use super::extra::remove_color_key;
}

use math::*;

pub const MAX_TEXTURE_SIZE: i32 = 4096;
