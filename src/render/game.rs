use super::*;
use crate::debug::DebugState;
use gfx::SpriteData;
use gfx2d::macroquad::prelude::*;
use ini::Ini;
use std::str::FromStr;

pub trait QuadGlProjectionExt {
    fn set_projection_from_transform(&mut self, transform: &Mat2d) -> Mat4;
    fn draw_batch(&mut self, slice: &mut DrawSlice, transform: &Mat2d);
}

impl QuadGlProjectionExt for QuadGl {
    fn set_projection_from_transform(&mut self, transform: &Mat2d) -> Mat4 {
        let prj = self.get_projection_matrix();

        self.set_projection_matrix(Mat4::from_cols_array_2d(&[
            [(transform.0).0, (transform.1).0, 0.0, 0.0],
            [(transform.0).1, (transform.1).1, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [(transform.0).2, (transform.1).2, 0.0, 1.0],
        ]));

        prj
    }

    fn draw_batch(&mut self, slice: &mut DrawSlice, transform: &Mat2d) {
        let buffer = slice.buffer();

        self.draw_mode(DrawMode::Triangles);

        let prj = self.set_projection_from_transform(transform);

        for cmd in slice.commands() {
            self.texture(
                cmd.texture
                    .map(|texture| Texture2D::from_miniquad_texture(texture)),
            );

            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            for i in cmd.vertex_range.clone() {
                let vert = buffer[i];

                indices.push(vertices.len() as u16);
                vertices.push(Vertex::new(
                    vert.pos.x,
                    vert.pos.y,
                    0.0,
                    vert.uv.x,
                    vert.uv.y,
                    [vert.color.r, vert.color.g, vert.color.b, vert.color.a].into(),
                ));
            }

            self.geometry(vertices.as_ref(), indices.as_ref());
        }

        self.set_projection_matrix(prj);
    }
}

pub struct GameGraphics {
    map: MapGraphics,
    soldier_graphics: SoldierGraphics,
    sprites: Vec<Vec<Sprite>>,
    batch: DrawBatch,
}

impl GameGraphics {
    pub fn new() -> GameGraphics {
        GameGraphics {
            map: MapGraphics::empty(),
            soldier_graphics: SoldierGraphics::new(),
            sprites: Vec::new(),
            batch: DrawBatch::new(),
        }
    }

    pub fn render_frame(
        &mut self,
        state: &MainState,
        debug_state: &DebugState,
        soldier: &Soldier,
        elapsed: f64,
        frame_percent: f32,
    ) {
        let zoom = f32::exp(state.zoom);
        let cam = lerp(state.camera_prev, state.camera, frame_percent);
        let (w, h) = (zoom * state.game_width, zoom * state.game_height);
        let (dx, dy) = (cam.x - w / 2.0, cam.y - h / 2.0);
        let transform = Transform::ortho(dx, dx + w, dy, dy + h).matrix();
        let transform_bg = Transform::ortho(0.0, 1.0, dy, dy + h).matrix();

        self.batch.clear();

        render_soldier(
            soldier,
            &self.soldier_graphics,
            &self.sprites,
            &mut self.batch,
            frame_percent,
        );

        if debug_state.render.render_skeleton {
            let px = h / screen_height();
            render_skeleton(soldier, &mut self.batch, px, frame_percent);
        }

        for bullet in &state.bullets {
            render_bullet(
                bullet,
                &self.sprites,
                &mut self.batch,
                elapsed,
                frame_percent,
            );
        }

        let macroquad::prelude::InternalGlContext { quad_gl: gl, .. } =
            unsafe { macroquad::prelude::get_internal_gl() };

        clear_background(BLACK);

        if !debug_state.render.disable_background {
            gl.draw_batch(&mut self.map.background(), &transform_bg);
        }
        if !debug_state.render.disable_polygon {
            if !debug_state.render.disable_texture {
                gl.draw_batch(&mut self.map.polys_back(), &transform);
            } else {
                // gl.draw_batch(&mut self.map.polys_back(), &transform);
            }
        }
        if !debug_state.render.disable_scenery_back {
            gl.draw_batch(&mut self.map.scenery_back(), &transform);
        }
        gl.draw_batch(&mut self.batch.all(), &transform);
        if !debug_state.render.disable_scenery_middle {
            gl.draw_batch(&mut self.map.scenery_mid(), &transform);
        }
        if !debug_state.render.disable_polygon {
            if !debug_state.render.disable_texture {
                gl.draw_batch(&mut self.map.polys_front(), &transform);
            } else {
                // gl.draw_batch(&mut self.map.polys_front(), &transform);
            }
        }
        if !debug_state.render.disable_scenery_front {
            gl.draw_batch(&mut self.map.scenery_front(), &transform);
        }

        if debug_state.ui_visible {
            let prj = gl.set_projection_from_transform(&transform);
            debug::debug_render(gl, debug_state, state, self);
            gl.set_projection_matrix(prj);
        }

        self.render_cursor(gl, state);
    }

    fn render_cursor(&mut self, gl: &mut QuadGl, state: &MainState) {
        let zoom = f32::exp(state.zoom);
        let (w, h) = (zoom * state.game_width, zoom * state.game_height);
        let size = vec2(screen_width(), screen_height());
        let x = zoom * f32::floor(state.mouse.x * size.x / w);
        let y = zoom * f32::floor(state.mouse.y * size.y / h);
        let screen = Transform::ortho(0.0, size.x, 0.0, size.y).matrix();

        self.batch.clear();

        self.batch.add_quad(
            None,
            &[
                vertex(vec2(x, y) + vec2(0.0, -8.0), Vec2::zero(), rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(1.0, -8.0), Vec2::zero(), rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(1.0, 9.0), Vec2::zero(), rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(0.0, 9.0), Vec2::zero(), rgb(0, 0, 0)),
            ],
        );

        self.batch.add_quad(
            None,
            &[
                vertex(vec2(x, y) + vec2(-8.0, 0.0), Vec2::zero(), rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(9.0, 0.0), Vec2::zero(), rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(9.0, 1.0), Vec2::zero(), rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(-8.0, 1.0), Vec2::zero(), rgb(0, 0, 0)),
            ],
        );

        gl.draw_batch(&mut self.batch.all(), &screen);
    }

    pub fn load_map(&mut self, fs: &mut Filesystem, map: &MapFile) {
        self.map = MapGraphics::new(fs, map);
    }

    pub fn load_sprites(&mut self, fs: &mut Filesystem) {
        let mut main: Vec<SpriteInfo> = Vec::new();
        let mut intf: Vec<SpriteInfo> = Vec::new();

        let add_to = |v: &mut Vec<SpriteInfo>, fname: &str| {
            let fname = filename_override(fs, "", fname);
            v.push(SpriteInfo::new(fname, vec2(1.0, 1.0), None));
        };

        for group in gfx::Group::values() {
            match *group {
                gfx::Group::Soldier => gfx::Soldier::values()
                    .iter()
                    .map(|v| v.filename())
                    .for_each(|f| add_to(&mut main, f)),

                gfx::Group::Weapon => gfx::Weapon::values()
                    .iter()
                    .map(|v| v.filename())
                    .for_each(|f| add_to(&mut main, f)),

                gfx::Group::Spark => gfx::Spark::values()
                    .iter()
                    .map(|v| v.filename())
                    .for_each(|f| add_to(&mut main, f)),

                gfx::Group::Object => gfx::Object::values()
                    .iter()
                    .map(|v| v.filename())
                    .for_each(|f| add_to(&mut main, f)),

                gfx::Group::Marker => gfx::Marker::values()
                    .iter()
                    .map(|v| v.filename())
                    .for_each(|f| add_to(&mut intf, f)),

                gfx::Group::Interface => gfx::Interface::values()
                    .iter()
                    .map(|v| v.filename())
                    .for_each(|f| add_to(&mut intf, f)),
            }
        }

        let mut file = fs.open("mod.ini").expect("Error opening File");
        if let Ok(cfg) = Ini::read_from(&mut file) {
            self.soldier_graphics.load_data(&cfg);

            if let Some(data) = cfg.section(Some("SCALE".to_owned())) {
                let default_scale = match data.get("DefaultScale") {
                    None => 1.0,
                    Some(scale) => f32::from_str(scale).unwrap_or(1.0),
                };

                for sprite_info in main.iter_mut().chain(intf.iter_mut()) {
                    let fname = sprite_info.filename.to_str().unwrap();

                    let scale = match data.get(fname) {
                        None => default_scale,
                        Some(scale) => f32::from_str(scale).unwrap_or(default_scale),
                    };

                    sprite_info.pixel_ratio = vec2(scale, scale);
                }
            }
        }

        let main = Spritesheet::new(fs, 8, FilterMode::Linear, &main);
        let intf = Spritesheet::new(fs, 8, FilterMode::Linear, &intf);

        self.sprites.clear();
        self.sprites.resize(gfx::Group::values().len(), Vec::new());

        let mut imain = 0;
        let mut iintf = 0;

        for group in gfx::Group::values() {
            let index = group.id();

            match *group {
                gfx::Group::Soldier => {
                    for _ in gfx::Soldier::values() {
                        self.sprites[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Weapon => {
                    for _ in gfx::Weapon::values() {
                        self.sprites[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Spark => {
                    for _ in gfx::Spark::values() {
                        self.sprites[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Object => {
                    for _ in gfx::Object::values() {
                        self.sprites[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Marker => {
                    for _ in gfx::Marker::values() {
                        self.sprites[index].push(intf.sprites[iintf].clone());
                        iintf += 1;
                    }
                }
                gfx::Group::Interface => {
                    for _ in gfx::Interface::values() {
                        self.sprites[index].push(intf.sprites[iintf].clone());
                        iintf += 1;
                    }
                }
            }
        }
    }

    pub fn get_sprite(&self, sprite: &dyn SpriteData) -> &gfx2d::Sprite {
        &self.sprites[sprite.group().id()][sprite.id()]
    }
}
