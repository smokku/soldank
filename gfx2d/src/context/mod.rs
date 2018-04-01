use super::*;
use gfx::Device;
use gfx::traits::Factory;
use gfx::traits::FactoryExt;
use glutin::GlContext;

mod pipeline;
use self::pipeline::{post, FRAG_SOURCE, VERT_SOURCE};
pub use self::pipeline::{pipe, Vertex};

pub fn vertex(pos: Vec2, texcoords: Vec2, color: Color) -> Vertex {
    Vertex {
        pos: [pos.x, pos.y],
        texcoords: [texcoords.x, texcoords.y],
        color: color.into(),
    }
}

impl ::std::default::Default for Vertex {
    fn default() -> Vertex {
        Vertex {
            pos: [0.0; 2],
            texcoords: [0.0; 2],
            color: [U8N(0); 4],
        }
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

        let ctx_b = glutin::ContextBuilder::new().with_vsync(true).with_gl(
            glutin::GlRequest::GlThenGles {
                opengl_version: (2, 1),
                opengles_version: (2, 0),
            },
        );

        let (wnd, dvc, mut fct, rtv_post, _) =
            ::gfx_window_glutin::init::<Srgba8, DepthStencil>(wnd_b, ctx_b, &evt);

        let mut enc = GlEncoder::from(fct.create_command_buffer());

        // main pipeline
        let (pso_main, rtv_main, rtv_main_sampler, white_texture) = {
            let pso = fct.create_pipeline_simple(VERT_SOURCE, FRAG_SOURCE, pipe::new());

            let render_target = fct.create_render_target::<Rgba8>(w as u16, h as u16);
            let (_, res_view, rtv) = render_target.unwrap();

            let sampler_info = SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp);
            let sampler = fct.create_sampler(sampler_info);

            let white_texture = texture::create_texture(
                &mut fct,
                &mut enc,
                (16, 16),
                &[255u8; 4 * 16 * 16],
                FilterMethod::Scale,
                WrapMode::Clamp,
            );

            (pso.unwrap(), rtv, (res_view, sampler), white_texture)
        };

        // post process pipeline
        let bundle_post = {
            let vtx = |pos, texcoords| post::Vertex { pos, texcoords };

            let quad = [
                vtx([-1.0, -1.0], [0.0, 0.0]),
                vtx([-1.0, 1.0], [0.0, 1.0]),
                vtx([1.0, 1.0], [1.0, 1.0]),
                vtx([1.0, 1.0], [1.0, 1.0]),
                vtx([1.0, -1.0], [1.0, 0.0]),
                vtx([-1.0, -1.0], [0.0, 0.0]),
            ];

            let (vs, fs, init) = (post::VERT_SOURCE, post::FRAG_SOURCE, post::pipe::new());
            let pso = fct.create_pipeline_simple(vs, fs, init).unwrap();
            let (vbuf, slice) = fct.create_vertex_buffer_with_slice(&quad, ());

            Bundle::new(
                slice,
                pso,
                post::pipe::Data {
                    vbuf,
                    sampler: rtv_main_sampler,
                    out: rtv_post,
                },
            )
        };

        Gfx2dContext {
            wnd,
            evt,
            fct,
            enc,
            dvc,
            rtv: rtv_main,
            pso: pso_main,
            bundle: bundle_post,
            white: white_texture,
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.enc.clear(
            &self.rtv,
            [
                f32::from(color.r()) / 255.0,
                f32::from(color.g()) / 255.0,
                f32::from(color.b()) / 255.0,
                f32::from(color.a()) / 255.0,
            ],
        );
    }

    pub fn draw(&mut self, slice: &mut DrawSlice, transform: &Mat2d) {
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
                Some(ref t) => t.handle(),
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
