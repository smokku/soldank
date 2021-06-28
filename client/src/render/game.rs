use super::*;
use gfx::SpriteData;
use gfx2d::macroquad::prelude::*;
use hocon::{Hocon, HoconLoader};
use ini::Ini;
use std::{collections::HashMap, io::Read, str::FromStr};

pub trait QuadGlProjectionExt {
    fn draw_batch(&mut self, slice: &mut DrawSlice);
}

impl QuadGlProjectionExt for QuadGl {
    fn draw_batch(&mut self, slice: &mut DrawSlice) {
        let buffer = slice.buffer();

        self.draw_mode(DrawMode::Triangles);

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
    }
}

pub struct GameGraphics {
    map: MapGraphics,
    soldier_graphics: SoldierGraphics,
    sprites: Vec<Vec<Sprite>>,
    batch: DrawBatch,
    dynamic_sprites: HashMap<String, HashMap<String, Sprite>>,
}

impl GameGraphics {
    pub fn new() -> GameGraphics {
        GameGraphics {
            map: MapGraphics::empty(),
            soldier_graphics: SoldierGraphics::new(),
            sprites: Vec::new(),
            batch: DrawBatch::new(),
            dynamic_sprites: HashMap::new(),
        }
    }

    pub fn render_frame(
        &mut self,
        resources: &Resources,
        soldier: &Soldier,
        elapsed: f64,
        frame_percent: f32,
    ) {
        let state = resources.get::<MainState>().unwrap();
        let config = resources.get::<Config>().unwrap();
        let bullets = resources.get::<Vec<Bullet>>().unwrap();
        let map = resources.get::<MapFile>().unwrap();

        let zoom = f32::exp(state.zoom);
        let cam = lerp(state.camera_prev, state.camera, frame_percent);
        let (w, h) = (zoom * state.game_width, zoom * state.game_height);
        let (dx, dy) = (cam.x - w / 2.0, cam.y - h / 2.0);

        let debug_state = &config.debug;

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

        for bullet in bullets.iter() {
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

        set_camera(&Camera2D::from_display_rect(Rect::new(0.0, dy, 1.0, h)));
        if !debug_state.render.disable_background {
            gl.draw_batch(&mut self.map.background());
        }

        set_camera(&Camera2D::from_display_rect(Rect::new(dx, dy, w, h)));

        if !debug_state.render.disable_polygon {
            if !debug_state.render.disable_texture {
                gl.draw_batch(&mut self.map.polys_back());
            } else {
                // gl.draw_batch(&mut self.map.polys_back());
            }
        }
        if !debug_state.render.disable_scenery_back {
            gl.draw_batch(&mut self.map.scenery_back());
        }
        gl.draw_batch(&mut self.batch.all());
        if !debug_state.render.disable_scenery_middle {
            gl.draw_batch(&mut self.map.scenery_mid());
        }
        if !debug_state.render.disable_polygon {
            if !debug_state.render.disable_texture {
                gl.draw_batch(&mut self.map.polys_front());
            } else {
                // gl.draw_batch(&mut self.map.polys_front());
            }
        }
        if !debug_state.render.disable_scenery_front {
            gl.draw_batch(&mut self.map.scenery_front());
        }

        if debug_state.visible {
            debug::debug_render(gl, debug_state, &*state, &*map, self);
        }

        set_default_camera();
        self.render_cursor(gl, &*state);
    }

    fn render_cursor(&mut self, gl: &mut QuadGl, state: &MainState) {
        let zoom = f32::exp(state.zoom);
        let (w, h) = (zoom * state.game_width, zoom * state.game_height);
        let size = vec2(screen_width(), screen_height());
        let x = zoom * f32::floor(state.mouse.x * size.x / w);
        let y = zoom * f32::floor(state.mouse.y * size.y / h);

        self.batch.clear();

        self.batch.add_quad(
            None,
            &[
                vertex(vec2(x, y) + vec2(0.0, -8.0), Vec2::ZERO, rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(1.0, -8.0), Vec2::ZERO, rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(1.0, 9.0), Vec2::ZERO, rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(0.0, 9.0), Vec2::ZERO, rgb(0, 0, 0)),
            ],
        );

        self.batch.add_quad(
            None,
            &[
                vertex(vec2(x, y) + vec2(-8.0, 0.0), Vec2::ZERO, rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(9.0, 0.0), Vec2::ZERO, rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(9.0, 1.0), Vec2::ZERO, rgb(0, 0, 0)),
                vertex(vec2(x, y) + vec2(-8.0, 1.0), Vec2::ZERO, rgb(0, 0, 0)),
            ],
        );

        gl.draw_batch(&mut self.batch.all());
    }

    pub fn load_map(&mut self, fs: &mut Filesystem, map: &MapFile) {
        self.map = MapGraphics::new(fs, map);
    }

    pub fn load_sprites(&mut self, fs: &mut Filesystem) {
        let mut main: Vec<SpriteInfo> = Vec::new();
        let mut intf: Vec<SpriteInfo> = Vec::new();
        let mut dynm: Vec<SpriteInfo> = Vec::new();

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

        let mut sprites_config = String::new();
        match fs.open("/sprites.conf") {
            Ok(mut file) => {
                if let Err(err) = file.read_to_string(&mut sprites_config) {
                    log::error!("Cannot read sprites.conf: {}", err);
                    std::process::abort();
                }
            }
            Err(err) => {
                log::error!("Cannot open sprites.conf: {}", err);
                std::process::abort();
            }
        }

        let mut loader = HoconLoader::new().no_system();
        loader = match loader.load_str(&sprites_config) {
            Ok(loader) => loader,
            Err(err) => {
                log::error!("Cannot load sprites.conf: {}", err);
                std::process::abort();
            }
        };

        let sprites_config = match loader.hocon() {
            Ok(hocon) => hocon,
            Err(err) => {
                log::error!("Cannot parse sprites.conf: {}", err);
                std::process::abort();
            }
        };
        log::trace!("Parsed sprites.conf: {:#?}", sprites_config);

        let groups = match sprites_config {
            Hocon::Hash(groups) => groups,
            _ => {
                log::error!("Error parsing sprites.conf groups: not a Hash");
                std::process::abort();
            }
        };

        let mut dynamic_sprites: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for group in groups.keys() {
            let sprites = match &groups[group] {
                Hocon::Hash(sprites) => sprites,
                _ => {
                    log::error!("Error parsing sprites.conf group {}: not a Hash", group);
                    std::process::abort();
                }
            };
            for sprite in sprites.keys() {
                let fname = match &sprites[sprite] {
                    Hocon::String(fname) => fname,
                    Hocon::Hash(data) => match &data["path"] {
                        Hocon::String(fname) => fname,
                        _ => {
                            log::error!(
                                "Error parsing sprites.conf sprite {}/{}: Missing 'path'",
                                group,
                                sprite
                            );
                            std::process::abort();
                        }
                    },
                    _ => {
                        log::error!("Error parsing sprites.conf sprite {}/{}", group, sprite);
                        std::process::abort();
                    }
                };
                dynamic_sprites
                    .entry((*group).clone())
                    .or_default()
                    .insert((*sprite).clone(), dynm.len());
                let fname = filename_override(fs, "", fname);
                dynm.push(SpriteInfo::new(fname, vec2(1.0, 1.0), None));
            }
        }

        let main = Spritesheet::new(fs, 8, FilterMode::Linear, &main);
        let intf = Spritesheet::new(fs, 8, FilterMode::Linear, &intf);
        let dynm = Spritesheet::new(fs, 8, FilterMode::Linear, &dynm);

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
                gfx::Group::Interface => {
                    for _ in gfx::Interface::values() {
                        self.sprites[index].push(intf.sprites[iintf].clone());
                        iintf += 1;
                    }
                }
            }
        }

        for group in dynamic_sprites.keys() {
            for (sprite, &index) in dynamic_sprites[group].iter() {
                self.dynamic_sprites
                    .entry((*group).clone())
                    .or_default()
                    .insert((*sprite).clone(), dynm.sprites[index].clone());
            }
        }
    }

    pub fn get_dynamic_sprite<S: Into<String>>(&self, group: S, sprite: S) -> &gfx2d::Sprite {
        let group = group.into();
        let sprite = sprite.into();
        &self
            .dynamic_sprites
            .get(&group)
            .expect(format!("Sprite group '{}' unavailable", group).as_str())
            .get(&sprite)
            .expect(format!("Sprite '{} / {}' unavailable", group, sprite).as_str())
    }
}
