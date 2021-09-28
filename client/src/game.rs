use crate::{
    constants::*,
    debug,
    engine::{input::Input, Engine, Game},
    mapfile::MapFile,
    render::{self as render, GameGraphics},
};
use gfx2d::math::Vec2;
use gvfs::filesystem::Filesystem;
use hecs::World;
use resources::Resources;

pub struct GameState {
    pub world: World,
    pub resources: Resources,
    pub filesystem: Filesystem,

    context: gfx2d::Gfx2dContext,
    graphics: GameGraphics,

    pub zoom: f32,
    pub mouse: Vec2,
}

impl GameState {
    pub(crate) fn new(
        context: gfx2d::Gfx2dContext,
        world: World,
        resources: Resources,
        filesystem: Filesystem,
    ) -> Self {
        GameState {
            context,
            graphics: GameGraphics::new(),
            world,
            resources,
            filesystem,
            zoom: 0.0,
            mouse: Vec2::default(),
        }
    }

    pub fn viewport(&self, camera: Vec2) -> (f32, f32, f32, f32) {
        let zoom = f32::exp(self.zoom);
        let (w, h) = (zoom * GAME_WIDTH, zoom * GAME_HEIGHT);
        let (dx, dy) = (camera.x - w / 2.0, camera.y - h / 2.0);
        (dx, dy, w, h)
    }

    pub fn mouse_to_world(&self, camera: Vec2, x: f32, y: f32) -> (f32, f32) {
        let (dx, dy, _w, _h) = self.viewport(camera);
        let zoom = f32::exp(self.zoom);
        (dx + x * zoom, dy + y * zoom)
    }
}

impl Game for GameState {
    fn initialize(&mut self, eng: Engine<'_>) {
        eng.quad_ctx.show_mouse(false);
        eng.quad_ctx.set_cursor_grab(true);

        let map = self.resources.get::<MapFile>().unwrap();
        self.graphics
            .load_sprites(eng.quad_ctx, &mut self.filesystem);
        self.graphics
            .load_map(eng.quad_ctx, &mut self.filesystem, &*map);
    }

    fn update(&mut self, eng: Engine<'_>) {
        if cfg!(debug_assertions) {
            debug::build_ui(&eng, self);
        }

        let screen_size = eng.quad_ctx.screen_size();
        self.mouse.x = eng.input.mouse_x * GAME_WIDTH / screen_size.0;
        self.mouse.y = eng.input.mouse_y * GAME_HEIGHT / screen_size.1;

        for _event in eng.input.drain_events() {
            // just drop it for now
        }
    }

    fn draw(&mut self, eng: Engine<'_>) {
        render::debug::debug_render(
            eng.quad_ctx,
            &mut self.graphics,
            &self.world,
            &self.resources,
        );

        self.graphics.render_frame(
            &mut self.context,
            eng.quad_ctx,
            &self.world,
            &self.resources,
            // self.last_frame - TIMESTEP_RATE * (1.0 - p),
            eng.overstep_percentage,
        );
    }
}
