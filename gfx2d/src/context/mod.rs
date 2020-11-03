use super::*;
use gfx::traits::Factory;
use gfx::traits::FactoryExt;
use gfx::Device;

mod pipeline;
pub use self::pipeline::{pipe, Vertex};
use self::pipeline::{post, FRAG_SOURCE, VERT_SOURCE};

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
    pub wnd_c: glutin::ContextWrapper<glutin::PossiblyCurrent, ()>,
    pub(crate) fct: GlFactory,
    pub(crate) enc: GlEncoder,
    dvc: GlDevice,
    rtv: Rgba8Target,
    pso: PipelineState,
    bundle: Bundle<R, post::pipe::Data<R>>,
    white: Texture,
}

impl Gfx2dContext {
    pub fn initialize(
        evt: glutin::event_loop::EventLoop<()>,
        title: &str,
        w: u32,
        h: u32,
    ) -> Gfx2dContext {
        let wnd_b = glutin::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(glutin::dpi::LogicalSize::new(w, h));

        let ctx_b =
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_gl(glutin::GlRequest::GlThenGles {
                    opengl_version: (2, 1),
                    opengles_version: (2, 0),
                });

        let gfx =
            ::gfx_window_glutin::init::<Srgba8, DepthStencil, ()>(wnd_b, ctx_b, &evt).unwrap();
        let wnd = unsafe { gfx.0.split() };
        Gfx2dContext::init_inner(
            w as gfx_core::texture::Size,
            h as gfx_core::texture::Size,
            wnd.0,
            gfx.1,
            gfx.2,
            gfx.3,
            gfx.4,
        )
    }

    // this is pulled-in internals of ::gfx_window_glutin::init_existing()
    pub fn initialize_existing(
        window: &glutin::window::Window,
        wnd_c: glutin::ContextWrapper<glutin::NotCurrent, ()>,
    ) -> Gfx2dContext {
        use gfx::format::Formatted;
        use gfx_core::{memory, texture};

        let wnd_c = unsafe { wnd_c.make_current().unwrap() };
        let (device, factory) =
            gfx_device_gl::create(|s| wnd_c.get_proc_address(s) as *const std::os::raw::c_void);

        // create the main color/depth targets
        let aa = wnd_c.get_pixel_format().multisampling.unwrap_or(0) as texture::NumSamples;
        let color_format = Srgba8::get_format();
        let ds_format = DepthStencil::get_format();
        let window_size = window.inner_size();
        let w = window_size.width as gfx_core::texture::Size;
        let h = window_size.height as gfx_core::texture::Size;
        let (color_view, ds_view) = gfx_device_gl::create_main_targets_raw(
            (w, h, 1, aa.into()),
            color_format.0,
            ds_format.0,
        );

        Gfx2dContext::init_inner(
            w,
            h,
            wnd_c,
            device,
            factory,
            memory::Typed::new(color_view),
            memory::Typed::new(ds_view),
        )
    }

    fn init_inner(
        w: gfx_core::texture::Size,
        h: gfx_core::texture::Size,
        wnd_c: glutin::ContextWrapper<glutin::PossiblyCurrent, ()>,
        dvc: gfx_device_gl::Device,
        mut fct: gfx_device_gl::Factory,
        rtv_post: gfx_core::handle::RenderTargetView<
            R,
            (gfx::format::R8_G8_B8_A8, gfx::format::Srgb),
        >,
        _dt_view: gfx_core::handle::DepthStencilView<R, (gfx::format::D24_S8, gfx::format::Unorm)>,
    ) -> Gfx2dContext {
        let mut enc = GlEncoder::from(fct.create_command_buffer());

        // main pipeline
        let (pso_main, rtv_main, rtv_main_sampler, white_texture) = {
            let pso = fct.create_pipeline_simple(VERT_SOURCE, FRAG_SOURCE, pipe::new());

            let render_target = fct.create_render_target::<Rgba8>(w, h);
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
            wnd_c,
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
        self.wnd_c.swap_buffers().unwrap();
        self.dvc.cleanup();
    }

    pub fn max_texture_size(&self) -> usize {
        self.dvc.get_capabilities().max_texture_size
    }
}
