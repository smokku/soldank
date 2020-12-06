#[macro_use]
extern crate gfx;

use {
    gfx::{
        buffer::Role::Vertex as VertexRole,
        format::{DepthStencil, Rgba8, Srgba8, U8Norm as U8N, R8_G8_B8_A8},
        memory::{Bind, Usage::Dynamic},
        pso::bundle::Bundle,
        state::{Blend, BlendChannel, BlendValue, ColorMask, Equation, Factor},
        texture::{AaMode, Kind::D2, Mipmap, SamplerInfo},
    },
    gfx_device_gl::{
        CommandBuffer as GlCommandBuffer, Device as GlDevice, Factory as GlFactory, Resources as R,
    },
};

// type aliases
type Rgba8Target = gfx::handle::RenderTargetView<R, Rgba8>;
type GlEncoder = gfx::Encoder<R, GlCommandBuffer>;
type PipelineState = gfx::PipelineState<R, context::pipe::Meta>;
type ShaderResourceView = gfx::handle::ShaderResourceView<R, [f32; 4]>;
type TextureHandle = gfx::handle::Texture<R, R8_G8_B8_A8>;
type Sampler = gfx::handle::Sampler<R>;
type VertexBuffer = gfx::handle::Buffer<R, Vertex>;

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
