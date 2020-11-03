// included from lib.rs
// traits are "used" where needed to avoid unused warnings (compiler bug?)

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
