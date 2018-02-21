pub mod renderer {
    use shared::state::*;
    use shared::sprites::*;
    use na::{Vector2};
    use gfx;
    use gfx::traits::FactoryExt;
    use gfx::Device;
    use gfx_window_glutin as gfx_glutin;
    use glutin::GlContext;
    pub type ColorFormat = gfx::format::Srgba8;

    pub type DepthFormat = gfx::format::Depth;
    use gfx::texture::Mipmap;
    use gfx::state::ColorMask;
    use gfx::Factory;
    use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
    use image;
    use glutin;
    use std::{thread, time};
    use std::path::PathBuf;
    use shared::mapfile::MapColor;

    const BLACK: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    gfx_defines!{
        vertex Vertex {
            pos: [f32; 3] = "position",
            tex_coords: [f32; 2] = "tex_coords",
            color: [f32; 4] = "color",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            transform: gfx::Global<[[f32; 3];3]> = "matrix",
            tex: gfx::TextureSampler<[f32; 4]> = "tex",
            out: gfx::BlendTarget<ColorFormat> = ("Target0", ColorMask::all(), gfx::preset::blend::ALPHA),
        }
        pipeline pipe_bg {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            transform: gfx::Global<[[f32; 3];3]> = "matrix",
            out: gfx::BlendTarget<ColorFormat> = ("Target0", ColorMask::all(), gfx::preset::blend::ALPHA),
        }
    }
    fn mat3ortho(l: f32, r: f32, t: f32, b: f32) -> [[f32; 3]; 3] {
        let w: f32 = r - l;
        let h: f32 = t - b;
        [
            [2.0 / w, 0.0, 0.0],
            [0.0, 2.0 / h, 0.0],
            [-(r + l) / w, -(t + b) / h, 1.00],
        ]
    }

    fn convert_color(color: MapColor) -> [f32; 4] {
        [
            f32::from(color.r) / 255.0,
            f32::from(color.g) / 255.0,
            f32::from(color.b) / 255.0,
            f32::from(color.a) / 255.0,
        ]
    }

    pub fn render(state: &mut MainState, sprite: &mut Sprite) {
        let mut events_loop = glutin::EventsLoop::new();
        let windowbuilder = glutin::WindowBuilder::new()
            .with_title("Soldank".to_string())
            .with_dimensions(1280, 720);
        let contextbuilder = glutin::ContextBuilder::new().with_vsync(true);
        let (window, mut device, mut factory, color_view, mut _depth_view) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(
                windowbuilder,
                contextbuilder,
                &events_loop,
            );

        window.window().set_cursor(glutin::MouseCursor::NoneCursor);
        window
            .window()
            .set_cursor_state(glutin::CursorState::Grab)
            .unwrap();

        let vs = include_bytes!("../../shaders/map_140.glslv");
        let ps = include_bytes!("../../shaders/map_140.glslf");
        let shader_set = factory.create_shader_set(vs, ps).unwrap();
        let pso_map = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::TriangleList,
                gfx::state::Rasterizer::new_fill(),
                pipe::new(),
            )
            .unwrap();
        let pso_bg = factory
            .create_pipeline_simple(
                include_bytes!("../../shaders/background_140.glsv"),
                include_bytes!("../../shaders/background_140.glsf"),
                pipe_bg::new(),
            )
            .unwrap();
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let mut map_polygon: Vec<Vertex> = Vec::new();
        for polygon in &mut state.map.polygons {
            for (_i, vertex) in polygon.vertices.iter().enumerate() {
                map_polygon.push(Vertex {
                    pos: [vertex.x, vertex.y, 1.0],
                    tex_coords: [vertex.u, vertex.v],
                    color: convert_color(vertex.color),
                });
            }
        }

        let (vertex_buffer_map, slice_map) = factory.create_vertex_buffer_with_slice(&map_polygon, ());
        let w = 768.0;
        let h = 480.0;

        let dx = 0.0 - w / 2.0;
        let dy = 0.0 - h / 2.0;
        let mut map_background: Vec<Vertex> = Vec::new();

        let d = 25.0 * state.map.sectors_division.max((0.5 * state.game_height as f32 / 25.0).ceil() as i32) as f32;

        map_background.push(Vertex {
            pos: [0.0, -d, 1.0],
            tex_coords: [0.0, 0.0],
            color: convert_color(state.map.bg_color_top),
        });
        map_background.push(Vertex {
            pos: [1.0, -d, 1.0],
            tex_coords: [0.0, 0.0],
            color: convert_color(state.map.bg_color_top),
        });
        map_background.push(Vertex {
            pos: [1.0, d, 1.0],
            tex_coords: [0.0, 0.0],
            color: convert_color(state.map.bg_color_bottom),
        });
        map_background.push(Vertex {
            pos: [0.0, d, 1.0],
            tex_coords: [0.0, 0.0],
            color: convert_color(state.map.bg_color_bottom),
        });

        let transform_bg = mat3ortho(0.00, 1.00, dy, h + dy);
        let indice: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let (vertex_buffer_bg, slice_bg) =
            factory.create_vertex_buffer_with_slice(&map_background, &indice[..]);

        fn gfx_load_texture<F, R>(
            factory: &mut F,
            file_name: &PathBuf,
        ) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
        where
            F: gfx::Factory<R>,
            R: gfx::Resources,
        {
            println!("Loading texture: {}", file_name.display());
            let img = image::open(file_name).unwrap().to_rgba();
            let (width, height) = img.dimensions();
            let kind =
                gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
            let (_, view) = factory
                .create_texture_immutable_u8::<ColorFormat>(kind, Mipmap::Provided, &[&img])
                .unwrap();
            view
        }
        let sampler =
            factory.create_sampler(SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Tile));

        let mut texture_file = PathBuf::new();
        texture_file.push("assets/textures");
        texture_file.push(state.map.texture_name.replace("bmp", "png"));
        if !texture_file.exists() {
            texture_file.set_extension("bmp");
        }
        let texture = gfx_load_texture(&mut factory, &texture_file);
        let mut data_map = pipe::Data {
            vbuf: vertex_buffer_map,
            transform: transform_bg,
            tex: (texture, sampler),
            out: color_view.clone(),
        };
        let mut data_bg = pipe_bg::Data {
            vbuf: vertex_buffer_bg,
            transform: transform_bg,
            out: color_view.clone(),
        };

        let mut closed = false;

        while !closed {
            let ten_millis = time::Duration::from_millis(10);

            thread::sleep(ten_millis);
            state.sprite_parts.do_eurler_timestep_for(1);
            sprite.update(state);

            state.camera_prev = state.camera;

            state.mouse_prev = state.mouse;

            let mut m = Vector2::new(0.0f32, 0.0f32);

            m.x = (state.mouse.x - state.game_width as f32 / 2.0) / 7.0
                * ((2.0 * 640.0 / state.game_width as f32 - 1.0)
                    + (state.game_width as f32 - 640.0) / state.game_width as f32 * 0.0
                        / 6.8);
            m.y = (state.mouse.y - state.game_width as f32 / 2.0) / 7.0;

            let mut cam_v = Vector2::new(state.camera.x, state.camera.y);

            let p = Vector2::new(state.sprite_parts.pos[1].x, state.sprite_parts.pos[1].y);
            let norm = p - cam_v;
            let s = norm * 0.14;
            cam_v += s;
            cam_v += m;

            state.camera = cam_v;
            let dx = state.camera.x - 768.00 / 2.0;
            let dy = state.camera.y - 480.00 / 2.0;

            let transform_map = mat3ortho(dx, w + dx, dy, h + dy);

            let pos = Vector2::new(state.sprite_parts.pos[1].x, state.sprite_parts.pos[1].y);

            let left = pos.x - 5.00;
            let right = pos.x + 5.00;
            let top = pos.y + 5.00;
            let bottom = pos.y - 10.00;
            let sprite_quad = vec![
                Vertex {
                    pos: [left, top, 1.0],
                    tex_coords: [0.0, 0.0],
                    color: convert_color(state.map.bg_color_bottom),
                },
                Vertex {
                    pos: [right, top, 1.0],
                    tex_coords: [0.0, 0.0],
                    color: convert_color(state.map.bg_color_bottom),
                },
                Vertex {
                    pos: [left, bottom, 1.0],
                    tex_coords: [0.0, 0.0],
                    color: convert_color(state.map.bg_color_bottom),
                },
                Vertex {
                    pos: [right, bottom, 1.0],
                    tex_coords: [0.0, 0.0],
                    color: convert_color(state.map.bg_color_bottom),
                },
            ];
            let indice_sprite: [u16; 6] = [0, 1, 2, 1, 3, 2];
            let (vertex_buffer_sprite, slice_sprite) =
                factory.create_vertex_buffer_with_slice(&sprite_quad, &indice_sprite[..]);
            let mut data_sprite = pipe_bg::Data {
                vbuf: vertex_buffer_sprite,
                transform: transform_map,
                out: color_view.clone(),
            };

            encoder.clear(&color_view, BLACK);

            data_map.transform = transform_map;
            data_bg.transform = transform_bg;
            data_sprite.transform = transform_map;

            encoder.draw(&slice_bg, &pso_bg, &data_bg);
            encoder.draw(&slice_map, &pso_map, &data_map);
            encoder.draw(&slice_sprite, &pso_bg, &data_sprite);
            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
            let mut mouse_inputs = Vec::new();
            events_loop.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        sprite.update_keys(&input);
                    }
                    glutin::WindowEvent::MouseInput { state, button, .. } => {
                        mouse_inputs.push((state, button));
                    }
                    glutin::WindowEvent::Closed => closed = true,
                    _ => (),
                },

                glutin::Event::DeviceEvent { event, .. } => match event {
                    glutin::DeviceEvent::MouseMotion { delta: (x, y) } => {
                        let game_width = state.game_width as f32;
                        let game_height = state.game_height as f32;
                        state.mouse.x = 0.0f32.max(game_width.min(state.mouse.x + x as f32 * 1.0));
                        state.mouse.y = 0.0f32.max(game_height.min(state.mouse.y + y as f32 * 1.0));
                    }
                    _ => (),
                },
                _ => (),
            });

            for mouse_input in &mouse_inputs {
                sprite.update_mouse_button(mouse_input);
            }
        }
    }

}
