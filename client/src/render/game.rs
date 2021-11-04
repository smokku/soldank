use super::*;
use crate::{constants::*, engine::world::WorldCameraExt};
use gfx::SpriteData;
use hocon::{Hocon, HoconLoader};
use ini::Ini;
use std::{collections::HashMap, convert::TryInto, io::Read, str::FromStr};

#[derive(Default)]
pub struct Sprites {
    stat: Vec<Vec<Sprite>>,
    dynamic: HashMap<String, HashMap<String, Sprite>>,
}

impl Sprites {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<S: Into<String>>(&self, group: S, sprite: S) -> &gfx2d::Sprite {
        let group = group.into();
        let sprite = sprite.into();

        if let Some(grp) = gfx::Group::values().iter().position(|g| g.name() == group) {
            if let Some(spr) = match gfx::Group::values()[grp] {
                gfx::Group::Soldier => gfx::Soldier::values()
                    .iter()
                    .position(|s| s.name() == sprite),
                gfx::Group::Weapon => gfx::Weapon::values()
                    .iter()
                    .position(|s| s.name() == sprite),
                gfx::Group::Spark => gfx::Spark::values().iter().position(|s| s.name() == sprite),
                gfx::Group::Object => gfx::Object::values()
                    .iter()
                    .position(|s| s.name() == sprite),
                gfx::Group::Interface => gfx::Interface::values()
                    .iter()
                    .position(|s| s.name() == sprite),
            } {
                return &self.stat[grp][spr];
            } else {
                panic!("Sprite '{} / {}' unavailable", group, sprite);
            }
        }

        self.dynamic
            .get(&group)
            .unwrap_or_else(|| panic!("Sprite group '{}' unavailable", group))
            .get(&sprite)
            .unwrap_or_else(|| panic!("Sprite '{} / {}' unavailable", group, sprite))
    }
}

pub struct GameGraphics {
    map: MapGraphics,
    soldier_graphics: SoldierGraphics,
    pub sprites: Sprites,
    batch: DrawBatch,
    debug_batch: DrawBatch,
}

impl GameGraphics {
    pub fn new() -> GameGraphics {
        GameGraphics {
            map: MapGraphics::empty(),
            soldier_graphics: SoldierGraphics::new(),
            sprites: Sprites::new(),
            batch: DrawBatch::new(),
            debug_batch: DrawBatch::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_frame(
        &mut self,
        context: &mut Gfx2dContext,
        ctx: &mut Context,
        world: &World,
        resources: &Resources,
        config: &Config,
        // soldier: &Soldier,
        // bullets: &[Bullet],
        // elapsed: f64,
        frame_percent: f32,
    ) {
        let (camera, camera_position) = world.get_camera_and_camera_position();

        let zoom = f32::exp(camera.zoom);
        let (w, h) = (zoom * GAME_WIDTH, zoom * GAME_HEIGHT);
        let mut cam = *camera_position;
        cam += camera.offset;
        if camera.centered {
            cam -= vec2(w / 2.0, h / 2.0);
        }
        // let cam = lerp(state.camera_prev, state.camera, frame_percent);
        // let (dx, dy) = (cam.x - w / 2.0, cam.y - h / 2.0);
        let transform = Transform::ortho(cam.x, cam.x + w, cam.y, cam.y + h).matrix();
        let transform_bg = Transform::ortho(0.0, 1.0, cam.y, cam.y + h).matrix();
        let phys_scale = config.phys.scale;

        let debug_state = &config.debug;

        // render_soldier(
        //     &*soldier,
        //     &self.soldier_graphics,
        //     &self.sprites.stat,
        //     &mut self.batch,
        //     frame_percent,
        // );

        // if debug_state.render.render_skeleton {
        //     let px = h / ctx.screen_size().1;
        //     render_skeleton(&*soldier, &mut self.debug_batch, px, frame_percent);
        // }

        // for bullet in bullets.iter() {
        //     render_bullet(
        //         bullet,
        //         &self.sprites.stat,
        //         &mut self.batch,
        //         elapsed,
        //         frame_percent,
        //     );
        // }

        ctx.begin_default_pass(mq::PassAction::clear_color(0.392, 0.584, 0.929, 1.0));

        if !debug_state.render.disable_background {
            context.draw(ctx, &mut self.map.background(), &transform_bg);
        }

        if !debug_state.render.disable_polygon {
            if !debug_state.render.disable_texture {
                context.draw(ctx, &mut self.map.polys_back(), &transform);
            } else {
                // draw using white texture
            }
        }
        if !debug_state.render.disable_scenery_back {
            context.draw(ctx, &mut self.map.scenery_back(), &transform);
        }
        render::systems::render_soldiers(
            world,
            &self.soldier_graphics,
            &self.sprites.stat,
            &mut self.batch,
            &mut self.debug_batch,
            frame_percent,
            h / ctx.screen_size().1,
            debug_state.render.render_skeleton,
        );
        render::systems::render_sprites(world, &self.sprites, &mut self.batch, phys_scale);
        context.draw(ctx, &mut self.batch.all(), &transform);
        if !debug_state.render.disable_scenery_middle {
            context.draw(ctx, &mut self.map.scenery_mid(), &transform);
        }
        if !debug_state.render.disable_polygon {
            if !debug_state.render.disable_texture {
                context.draw(ctx, &mut self.map.polys_front(), &transform);
            } else {
                // draw using white texture
            }
        }
        if !debug_state.render.disable_scenery_front {
            context.draw(ctx, &mut self.map.scenery_front(), &transform);
        }
        ctx.end_render_pass();
        self.batch.clear();

        if debug_state.visible {
            context.draw(ctx, &mut self.debug_batch.all(), &transform);
        }
        self.debug_batch.clear();

        // UI pass
        let screen = Transform::ortho(0.0, GAME_WIDTH, 0.0, GAME_HEIGHT).matrix();
        ctx.begin_default_pass(mq::PassAction::Nothing);
        render::systems::render_cursor(world, &self.sprites, &mut self.batch);
        context.draw(ctx, &mut self.batch.all(), &screen);
        ctx.end_render_pass();
        self.batch.clear();
    }

    pub fn load_map(&mut self, ctx: &mut Context, fs: &mut Filesystem, map: &MapFile) {
        self.map = MapGraphics::new(ctx, fs, map);
    }

    pub fn load_sprites(&mut self, ctx: &mut Context, fs: &mut Filesystem) {
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

        let groups = match &sprites_config {
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
                    .insert((*sprite).clone(), main.len());
                let fname = filename_override(fs, "", fname);
                main.push(SpriteInfo::new(fname, vec2(1.0, 1.0), None));
            }
        }

        let main = Spritesheet::new(ctx, fs, 8, FilterMode::Linear, &main);
        let intf = Spritesheet::new(ctx, fs, 8, FilterMode::Linear, &intf);

        self.sprites.stat.clear();
        self.sprites
            .stat
            .resize(gfx::Group::values().len(), Vec::new());

        let mut imain = 0;
        let mut iintf = 0;

        for group in gfx::Group::values() {
            let index = group.id();

            match *group {
                gfx::Group::Soldier => {
                    for _ in gfx::Soldier::values() {
                        self.sprites.stat[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Weapon => {
                    for _ in gfx::Weapon::values() {
                        self.sprites.stat[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Spark => {
                    for _ in gfx::Spark::values() {
                        self.sprites.stat[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Object => {
                    for _ in gfx::Object::values() {
                        self.sprites.stat[index].push(main.sprites[imain].clone());
                        imain += 1;
                    }
                }
                gfx::Group::Interface => {
                    for _ in gfx::Interface::values() {
                        self.sprites.stat[index].push(intf.sprites[iintf].clone());
                        iintf += 1;
                    }
                }
            }
        }

        for group in dynamic_sprites.keys() {
            for (spr, &index) in dynamic_sprites[group].iter() {
                let mut sprite = main.sprites[index].clone();
                if let Hocon::Hash(spr) = &sprites_config[group.as_str()][spr.as_str()] {
                    if let Hocon::Integer(width) = spr["width"] {
                        sprite.width = width as f32;
                    }
                    if let Hocon::Integer(height) = spr["height"] {
                        sprite.height = height as f32;
                    }
                }
                self.sprites
                    .dynamic
                    .entry((*group).clone())
                    .or_default()
                    .insert((*spr).clone(), sprite);
            }
        }
    }

    pub fn add_debug_geometry(&mut self, texture: Option<&Texture>, vertices: &[Vertex]) {
        if vertices.len() % 4 == 0 {
            for chunk in vertices.chunks_exact(4) {
                self.debug_batch
                    .add_quad(texture, chunk.try_into().unwrap());
            }
        } else if vertices.len() % 3 == 0 {
            for chunk in vertices.chunks_exact(3) {
                self.debug_batch.add(texture, chunk.try_into().unwrap());
            }
        } else {
            panic!(
                "cannot render debug geometry vertices count {}",
                vertices.len()
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_debug_line<C: Into<Color>>(
        &mut self,
        x1: f32,
        y1: f32,
        color1: C,
        x2: f32,
        y2: f32,
        color2: C,
        thickness: f32,
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let v = vec2(dx, dy);
        if let Some(t) = v.perp().try_normalize() {
            let t = t * (thickness / 2.);
            let p1 = vec2(x1, y1);
            let p2 = vec2(x2, y2);
            let color1 = color1.into();
            let color2 = color2.into();

            self.add_debug_geometry(
                None,
                &[
                    vertex(p1 + t, vec2(0., 0.), color1),
                    vertex(p1 - t, vec2(0., 0.), color1),
                    vertex(p2 - t, vec2(0., 0.), color2),
                    vertex(p2 + t, vec2(0., 0.), color2),
                ],
            );
        }
    }

    pub fn draw_debug_polyline<C: Into<Color> + Copy>(
        &mut self,
        vertices: &[(f32, f32, C)],
        thickness: f32,
    ) {
        for (i, &vert) in vertices.iter().enumerate() {
            let next = &vertices[(i + 1) % vertices.len()];
            self.draw_debug_line(vert.0, vert.1, vert.2, next.0, next.1, next.2, thickness);
        }
    }

    pub fn draw_debug_sprite<S: Into<String>>(
        &mut self,
        group: S,
        sprite: S,
        x: f32,
        y: f32,
        hwidth: f32,
        hheight: f32,
    ) {
        let group = group.into();
        let sprite = sprite.into();
        let (texture, tx, ty) = if group.is_empty() || sprite.is_empty() {
            (None, (0., 0.), (0., 0.))
        } else {
            let sprite = self.sprites.get(group, sprite);
            (sprite.texture, sprite.texcoords_x, sprite.texcoords_y)
        };

        self.add_debug_geometry(
            texture.as_ref(),
            &[
                vertex(
                    vec2(x - hwidth, y - hheight),
                    vec2(tx.0, ty.0),
                    rgb(255, 255, 255),
                ),
                vertex(
                    vec2(x + hwidth, y - hheight),
                    vec2(tx.1, ty.0),
                    rgb(255, 255, 255),
                ),
                vertex(
                    vec2(x + hwidth, y + hheight),
                    vec2(tx.1, ty.1),
                    rgb(255, 255, 255),
                ),
                vertex(
                    vec2(x - hwidth, y + hheight),
                    vec2(tx.0, ty.1),
                    rgb(255, 255, 255),
                ),
            ],
        );
    }

    fn get_circle_vertices(x: f32, y: f32, radius: f32, rotation: Rad) -> Vec<Vec2> {
        const STEPS: usize = 16;
        let pos = vec2(x, y);
        let mut vertices = Vec::with_capacity(STEPS);
        for step in 0..STEPS {
            let m = Transform::FromOrigin {
                pos,
                scale: vec2(1.0, 1.0),
                rot: (
                    rotation + (2. * PI / STEPS as f32) * step as f32,
                    Vec2::ZERO,
                ),
            }
            .matrix();

            vertices.push(m * vec2(radius, 0.0));
        }
        vertices
    }

    pub fn draw_debug_disk<C: Into<Color>>(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        rotation: Rad,
        color_c: C,
        color_r: C,
    ) {
        let color_c = color_c.into();
        let color_r = color_r.into();
        let vertices = Self::get_circle_vertices(x, y, radius, rotation);
        for (i, &vert) in vertices.iter().enumerate() {
            let next = vertices[(i + 1) % vertices.len()];
            self.add_debug_geometry(
                None,
                &[
                    vertex(vec2(x, y), Vec2::ZERO, color_c),
                    vertex(vert, Vec2::ZERO, color_r),
                    vertex(next, Vec2::ZERO, color_r),
                ],
            );
        }
    }

    pub fn draw_debug_circle<C: Into<Color> + Copy>(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        rotation: Rad,
        color: C,
        thickness: f32,
    ) {
        let vertices: Vec<(f32, f32, C)> = Self::get_circle_vertices(x, y, radius, rotation)
            .iter()
            .map(|vertex| (vertex.x, vertex.y, color))
            .collect();
        self.draw_debug_polyline(vertices.as_slice(), thickness);
    }

    pub fn draw_debug_half_circle<C: Into<Color> + Copy>(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        rotation: Rad,
        color: C,
        thickness: f32,
    ) {
        let vertices: Vec<(f32, f32, C)> = Self::get_circle_vertices(x, y, radius, rotation)
            .iter()
            .map(|vertex| (vertex.x, vertex.y, color))
            .collect();
        for (i, &vert) in vertices.iter().enumerate().take(vertices.len() / 2) {
            let next = &vertices[(i + 1) % vertices.len()];
            self.draw_debug_line(vert.0, vert.1, vert.2, next.0, next.1, next.2, thickness);
        }
    }
}
