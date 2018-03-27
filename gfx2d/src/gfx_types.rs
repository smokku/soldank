// included from lib.rs
// traits are "used" where needed to avoid unused warnings (compiler bug?)

use glutin::GlWindow;
use glutin::EventsLoop;
use gfx::pso::bundle::Bundle;
use gfx::buffer::Role::Vertex as VertexRole;
use gfx::memory::Bind;
use gfx::memory::Usage::Dynamic;
use gfx::format::Rgba8;
use gfx::format::Srgba8;
use gfx::format::R8_G8_B8_A8;
use gfx::format::DepthStencil;
use gfx::format::U8Norm as U8N;
use gfx::texture::AaMode;
use gfx::texture::Kind::D2;
use gfx::texture::SamplerInfo;
use gfx::texture::Mipmap;
use gfx::state::ColorMask;
use gfx::state::Blend;
use gfx::state::BlendChannel;
use gfx::state::BlendValue;
use gfx::state::Equation;
use gfx::state::Factor;
use gfx_device_gl::Resources as R;
use gfx_device_gl::Device as GlDevice;
use gfx_device_gl::Factory as GlFactory;
use gfx_device_gl::CommandBuffer as GlCommandBuffer;

// type aliases

type Rgba8Target = gfx::handle::RenderTargetView<R, Rgba8>;
type GlEncoder = gfx::Encoder<R, GlCommandBuffer>;
type PipelineState = gfx::PipelineState<R, context::pipe::Meta>;
type ShaderResourceView = gfx::handle::ShaderResourceView<R, [f32; 4]>;
type TextureHandle = gfx::handle::Texture<R, R8_G8_B8_A8>;
type Sampler = gfx::handle::Sampler<R>;
type VertexBuffer = gfx::handle::Buffer<R, Vertex>;
