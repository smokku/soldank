use crate::{
    cvars::Config, engine::world::WorldCameraExt, engine::Engine, game::GameState,
    render::components::Cursor,
};
use cvar::{INode, IVisit};
pub use gfx2d::math::*;
use hecs::World;

// mod entities;
mod cli;
mod render;
mod spawner;

pub use render::RenderState;

#[derive(Default)]
pub struct DebugState {
    pub visible: bool,
    cli: cli::CliState,
    spawner: spawner::SpawnerState,
    pub render: RenderState,
}

impl IVisit for DebugState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
        f(&mut cvar::List("cli", &mut self.cli));
        f(&mut cvar::List("spawner", &mut self.spawner));
        f(&mut cvar::List("render", &mut self.render));
    }
}

pub fn build_ui(eng: &Engine<'_>, game: &mut GameState) {
    let mut config = game.resources.get_mut::<Config>().unwrap();

    // if config.debug.fps_second != seconds_since_startup {
    //     config.debug.fps = config.debug.fps_count;
    //     config.debug.fps_second = seconds_since_startup;
    //     config.debug.fps_count = 0;
    // }
    // config.debug.fps_count += 1;

    if config.debug.visible {
        let mouse = if let Some((_entity, cursor)) = game.world.query::<&Cursor>().iter().next() {
            **cursor
        } else {
            Vec2::ZERO
        };
        let scale = config.phys.scale;
        let (camera, camera_position) = game.world.get_camera_and_camera_position();
        let (dx, dy, _w, _h) = camera.viewport(*camera_position);
        let (x, y) = camera.mouse_to_world(*camera_position, mouse.x, mouse.y);

        egui::Window::new("Debugger")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .show(eng.egui_ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    toggle_state(ui, &mut config.debug.cli.visible, "CLI");
                    toggle_state(ui, &mut config.debug.spawner.visible, "Spawn");
                    toggle_state(
                        ui, &mut false, /*config.debug.entities.visible*/ "Entities",
                    );
                    toggle_state(ui, &mut config.debug.render.visible, "Render");
                });

                ui.separator();
                ui.scope(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                    ui.label(format!(
                        "{:4}FPS \u{B1}{}",
                        eng.fps, eng.overstep_percentage
                    ));

                    ui.horizontal_wrapped(|ui| {
                        if ui.button("\u{2196}").clicked() {
                            eng.quad_ctx.set_cursor_grab(false);
                        }
                        ui.label(format!(
                            "{:4} {:3} [{:.3} {:.3}]",
                            eng.input.mouse_x, eng.input.mouse_y, mouse.x, mouse.y
                        ));
                    });
                    ui.label(format!(
                        " \u{1F5FA} {:4.3} {:3.3} ({:.3},{:.3})",
                        x, y, dx, dy
                    ));
                });
            });

        // config.debug.cli.build_ui();
        config
            .debug
            .spawner
            .build_ui(eng.egui_ctx, &mut game.world, x, y, scale);
        // config.debug.entities.build_ui();
        config.debug.render.build_ui(eng.egui_ctx);
    }
}

fn toggle_state(ui: &mut egui::Ui, state: &mut bool, label: &str) {
    if ui.selectable_label(*state, label).clicked() {
        *state = !*state;
    }
}

fn toggle_state_inv(ui: &mut egui::Ui, state: &mut bool, label: &str) {
    if ui.selectable_label(!*state, label).clicked() {
        *state = !*state;
    }
}
