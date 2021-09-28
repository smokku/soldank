use crate::{
    debug,
    engine::{Engine, Game},
    mapfile::MapFile,
    render::{self as render, GameGraphics},
};
use gvfs::filesystem::Filesystem;
use hecs::World;
use resources::Resources;

pub struct GameState {
    world: World,
    resources: Resources,
    filesystem: Filesystem,

    context: gfx2d::Gfx2dContext,
    graphics: GameGraphics,
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
        }
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
            debug::build_ui(
                eng.quad_ctx,
                eng.egui_ctx,
                &mut self.world,
                &self.resources,
                eng.fps,
                eng.overstep_percentage,
            );
        }

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
