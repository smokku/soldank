use super::*;
use cvar::{INode, IVisit};
// use macroquad::ui::{hash, root_ui, widgets, Ui};

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

    pub fps: u16,
    fps_second: u64,
    fps_count: u16,
}

impl IVisit for DebugState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
        f(&mut cvar::List("cli", &mut self.cli));
        f(&mut cvar::List("spawner", &mut self.spawner));
        f(&mut cvar::List("render", &mut self.render));
        f(&mut cvar::ReadOnlyProp("fps", &self.fps, 0));
    }
}

pub fn build_ui(
    ctx: &mut mq::Context,
    egui_ctx: &egui::CtxRef,
    world: &mut World,
    resources: &Resources,
    seconds_since_startup: u64,
    overstep_percentage: f32,
) {
    let game = resources.get::<MainState>().unwrap();
    let mut config = resources.get_mut::<Config>().unwrap();
    let scale = config.phys.scale;

    if config.debug.fps_second != seconds_since_startup {
        config.debug.fps = config.debug.fps_count;
        config.debug.fps_second = seconds_since_startup;
        config.debug.fps_count = 0;
    }
    config.debug.fps_count += 1;

    if config.debug.visible {
        let (dx, dy, _w, _h) = game.viewport(1.0);
        let (x, y) = game.mouse_to_world(1.0, game.mouse.x, game.mouse.y);

        egui::Window::new("Egui Window")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .show(egui_ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .selectable_label(config.debug.cli.visible, "CLI")
                        .clicked()
                    {
                        config.debug.cli.visible = !config.debug.cli.visible;
                    }
                    if ui
                        .selectable_label(config.debug.spawner.visible, "Spawn")
                        .clicked()
                    {
                        config.debug.spawner.visible = !config.debug.spawner.visible;
                    }
                    if ui
                        .selectable_label(false, /*config.debug.entities.visible*/ "Entities")
                        .clicked()
                    {
                        // config.debug.entities.visible = !config.debug.entities.visible;
                    }
                    if ui
                        .selectable_label(config.debug.render.visible, "Render")
                        .clicked()
                    {
                        config.debug.render.visible = !config.debug.render.visible;
                    }
                });

                ui.separator();
                ui.scope(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                    ui.label(format!(
                        "{:4}FPS \u{B1}{}",
                        config.debug.fps, overstep_percentage
                    ));

                    ui.horizontal_wrapped(|ui| {
                        if ui.button("\u{2196}").clicked() {
                            ctx.set_cursor_grab(false);
                        }
                        ui.label(format!(
                            "{:4} {:3} [{:.3} {:.3}]",
                            game.mouse_phys.x as u32,
                            game.mouse_phys.y as u32,
                            game.mouse.x,
                            game.mouse.y
                        ));
                    });
                    ui.label(format!(
                        " \u{1F5FA} {:4.3} {:3.3} ({:.3},{:.3})",
                        x, y, dx, dy
                    ));
                });
            });

        // config.debug.cli.build_ui();
        // config.debug.spawner.build_ui(world, x, y, scale);
        // // config.debug.entities.build_ui();
        // config.debug.render.build_ui();
    }
}
