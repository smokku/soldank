mod batch;
mod color;
mod context;
mod extra;
mod spritesheet;
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
pub use miniquad::{
    self as mq, Buffer as VertexBuffer, Context, FilterMode, Texture, TextureAccess, TextureFormat,
    TextureParams, TextureWrap,
};
pub use spritesheet::Sprite;
pub use spritesheet::SpriteInfo;
pub use spritesheet::Spritesheet;
pub use transform::Transform;

pub mod gfx2d_extra {
    pub use super::extra::load_image_rgba;
    pub use super::extra::premultiply_image;
    pub use super::extra::remove_color_key;
}

pub use image;
use math::*;

pub const MAX_TEXTURE_SIZE: i32 = 4096;
