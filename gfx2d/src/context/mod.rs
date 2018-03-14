use super::*;
use glutin::GlContext;
use gfx::Device;
use gfx::traits::FactoryExt;

mod pipeline;
use self::pipeline::{VERT_SOURCE, FRAG_SOURCE};
pub use self::pipeline::{Vertex, pipe};

pub fn vertex(pos: Vec2, texcoords: Vec2, color: Color) -> Vertex {
    Vertex {
        pos: [pos.x, pos.y],
        texcoords: [texcoords.x, texcoords.y],
        color: color.into()
    }
}

// Gfx2dContext

pub struct Gfx2dContext {
    pub wnd: GlWindow,
    pub evt: EventsLoop,
    pub(crate) fct: GlFactory,
    pub(crate) enc: GlEncoder,
    dvc: GlDevice,
    rtv: RenderTargetView,
    pso: PipelineState,
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

        let (wnd, dvc, mut fct, rtv, _) =
            ::gfx_window_glutin::init::<ColorFormat, DepthStencil>(wnd_b, ctx_b, &evt);

        let pso = fct.create_pipeline_simple(VERT_SOURCE, FRAG_SOURCE, pipe::new()).unwrap();
        let mut enc = GlEncoder::from(fct.create_command_buffer());
        let white = texture::create_texture(&mut fct, &mut enc, (16, 16), &[255u8; 4*16*16],
            FilterMethod::Scale, WrapMode::Clamp);

        // loading cursor fix
        wnd.set_cursor_state(glutin::CursorState::Hide).unwrap();
        wnd.set_cursor_state(glutin::CursorState::Normal).unwrap();

        Gfx2dContext {wnd, evt, fct, enc, dvc, rtv, pso, white}
    }

    pub fn clear(&mut self, color: Color) {
        self.enc.clear(&self.rtv, [
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            color.a() as f32 / 255.0
        ]);
    }

    pub fn draw(&mut self, batch_slice: BatchSlice, transform: &Mat2d) {
        batch_slice.batch.update(&mut self.fct, &mut self.enc);

        let mut data = pipe::Data {
            vbuf: batch_slice.buffer(),
            transform: transform.to_3x3(),
            sampler: self.white.handle(),
            out: self.rtv.clone(),
        };

        for cmd in batch_slice.commands() {
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
        self.enc.flush(&mut self.dvc);
        self.wnd.swap_buffers().unwrap();
        self.dvc.cleanup();
    }

    pub fn max_texture_size(&self) -> usize {
        self.dvc.get_capabilities().max_texture_size
    }
}
