use crate::{
    engine::world::WorldCameraExt, engine::Engine, game::GameState, render::components::Cursor,
};
use cvar::{INode, IVisit};
pub use gfx2d::math::*;
use hecs::World;

mod cli;
mod entities;
mod render;
mod spawner;

pub use render::RenderState;

#[derive(Default)]
pub struct DebugState {
    pub visible: bool,
    cli: cli::CliState,
    spawner: spawner::SpawnerState,
    entities: entities::EntitiesState,
    pub render: RenderState,
}

impl IVisit for DebugState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
        f(&mut cvar::List("cli", &mut self.cli));
        f(&mut cvar::List("spawner", &mut self.spawner));
        f(&mut cvar::List("entities", &mut self.entities));
        f(&mut cvar::List("render", &mut self.render));
    }
}

pub fn build_ui(eng: &Engine<'_>, game: &mut GameState) {
    let gravity = game.config.phys.gravity;
    let debug = &mut game.config.debug;

    if debug.visible {
        let mouse = if let Some((_entity, cursor)) = game.world.query::<&Cursor>().iter().next() {
            **cursor
        } else {
            Vec2::ZERO
        };
        let scale = game.config.phys.scale;
        let (camera, camera_position) = game.world.get_camera_and_camera_position();
        let (dx, dy, _w, _h) = camera.viewport(*camera_position);
        let (x, y) = camera.mouse_to_world(*camera_position, mouse.x, mouse.y);

        egui::Window::new("Debugger")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .show(eng.egui_ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(debug.cli.visible, "CLI").clicked() {
                        debug.cli.visible = !debug.cli.visible;
                        debug.cli.auto_focus = true;
                        debug.cli.auto_scroll = true;
                    }
                    toggle_state(ui, &mut debug.spawner.visible, "Spawn");
                    toggle_state(ui, &mut debug.entities.visible, "Entities");
                    toggle_state(ui, &mut debug.render.visible, "Render");
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

        debug.cli.build_ui(eng);
        debug
            .spawner
            .build_ui(eng.egui_ctx, &mut game.world, x, y, scale, gravity);
        debug.entities.build_ui(eng.egui_ctx, &mut game.world);
        debug.render.build_ui(eng.egui_ctx);
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
