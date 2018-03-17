// included from lib.rs
// traits are "used" where needed to avoid unused warnings (compiler bug?)

use {
    glutin::{
        GlWindow,
        EventsLoop,
    },
    gfx::{
        pso::bundle::Bundle,
        buffer::Role::Vertex as VertexRole,
        memory::{
            Bind,
            Usage::Dynamic,
        },
        format::{
            Rgba8,
            Srgba8,
            R8_G8_B8_A8,
            DepthStencil,
            U8Norm as U8N,
        },
        texture::{
            AaMode,
            Kind::D2,
            SamplerInfo,
            Mipmap,
        },
        state::{
            ColorMask,
            Blend,
            BlendChannel,
            BlendValue,
            Equation,
            Factor,
        },
    },
    gfx_device_gl::{
        Resources as R,
        Device as GlDevice,
        Factory as GlFactory,
        CommandBuffer as GlCommandBuffer,
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
