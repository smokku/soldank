use super::*;
use glutin::GlContext;
use gfx::Device;
use gfx::traits::Factory;
use gfx::traits::FactoryExt;

mod pipeline;
use self::pipeline::{post, VERT_SOURCE, FRAG_SOURCE};
pub use self::pipeline::{Vertex, pipe};

pub fn vertex(pos: Vec2, texcoords: Vec2, color: Color) -> Vertex {
    Vertex {
        pos: [pos.x, pos.y],
        texcoords: [texcoords.x, texcoords.y],
        color: color.into()
    }
}

impl ::std::default::Default for Vertex {
    fn default() -> Vertex {
        Vertex { pos: [0.0; 2], texcoords: [0.0; 2], color: [U8N(0); 4] }
    }
}

// Gfx2dContext

pub struct Gfx2dContext {
    pub wnd: GlWindow,
    pub evt: EventsLoop,
    pub(crate) fct: GlFactory,
    pub(crate) enc: GlEncoder,
    dvc: GlDevice,
    rtv: Rgba8Target,
    pso: PipelineState,
    bundle: Bundle<R, post::pipe::Data<R>>,
    white: Texture,
}

impl Gfx2dContext {
    pub fn initialize(title: &str, w: u32, h: u32) -> Gfx2dContext {
        let evt = glutin::EventsLoop::new();

        let wnd_b = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(w, h);

        let ctx_b = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_gl(glutin::GlRequest::GlThenGles{
                opengl_version: (2, 1),
                opengles_version: (2, 0)
            });

        let (wnd, dvc, mut fct, rtv_post, _) =
            ::gfx_window_glutin::init::<Srgba8, DepthStencil>(wnd_b, ctx_b, &evt);

        let quad = [
            post::Vertex{pos:[-1.0, -1.0], texcoords:[0.0, 0.0]},
            post::Vertex{pos:[-1.0,  1.0], texcoords:[0.0, 1.0]},
            post::Vertex{pos:[ 1.0,  1.0], texcoords:[1.0, 1.0]},
            post::Vertex{pos:[ 1.0,  1.0], texcoords:[1.0, 1.0]},
            post::Vertex{pos:[ 1.0, -1.0], texcoords:[1.0, 0.0]},
            post::Vertex{pos:[-1.0, -1.0], texcoords:[0.0, 0.0]},
        ];

        let mut enc = GlEncoder::from(fct.create_command_buffer());
        let pso = fct.create_pipeline_simple(VERT_SOURCE, FRAG_SOURCE, pipe::new()).unwrap();
        let pso_post = fct.create_pipeline_simple(post::VERT_SOURCE, post::FRAG_SOURCE, post::pipe::new()).unwrap();
        let (_, v, rtv) = fct.create_render_target::<Rgba8>(w as u16, h as u16).unwrap();
        let (vbuf, slice) = fct.create_vertex_buffer_with_slice(&quad, ());

        let bundle = Bundle::new(slice, pso_post, post::pipe::Data {
            vbuf,
            sampler: (v, fct.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp))),
            out: rtv_post,
        });

        let white = texture::create_texture(&mut fct, &mut enc, (16, 16), &[255u8; 4*16*16],
            FilterMethod::Scale, WrapMode::Clamp);

        // loading cursor fix
        wnd.set_cursor_state(glutin::CursorState::Hide).unwrap();
        wnd.set_cursor_state(glutin::CursorState::Normal).unwrap();

        Gfx2dContext {wnd, evt, fct, enc, dvc, rtv, pso, bundle, white}
    }

    pub fn clear(&mut self, color: Color) {
        self.enc.clear(&self.rtv, [
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            color.a() as f32 / 255.0
        ]);
    }

    pub fn draw(&mut self, slice: DrawSlice, transform: &Mat2d) {
        slice.batch.update(self);

        let mut data = pipe::Data {
            vbuf: slice.buffer(),
            transform: transform.to_3x3(),
            sampler: self.white.handle(),
            out: self.rtv.clone(),
        };

        for cmd in slice.commands() {
            let slice = gfx::Slice {
                start: cmd.vertex_range.start as u32,
                end: cmd.vertex_range.end as u32,
                base_vertex: 0,
                instances: None,
                buffer: gfx::IndexBuffer::Auto,
            };

            data.sampler = match cmd.texture {
                None => self.white.handle(),
                Some(ref t) => t.handle()
            };

            self.enc.draw(&slice, &self.pso, &data);
        }
    }

    pub fn present(&mut self) {
        self.bundle.encode(&mut self.enc);
        self.enc.flush(&mut self.dvc);
        self.wnd.swap_buffers().unwrap();
        self.dvc.cleanup();
    }

    pub fn max_texture_size(&self) -> usize {
        self.dvc.get_capabilities().max_texture_size
    }
}
