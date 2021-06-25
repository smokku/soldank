use super::*;
use cvar::{INode, IVisit};
use macroquad::ui::{hash, root_ui, widgets};

// mod entities;
mod render;
// mod spawner;

pub use render::RenderState;

#[derive(Default)]
pub struct DebugState {
    pub ui_visible: bool,
    spawner_visible: bool,
    // spawner: spawner::SpawnerState,
    entities_visible: bool,
    render_visible: bool,
    pub render: RenderState,

    pub fps: u32,
    fps_second: u32,
    fps_count: u32,
}

impl IVisit for DebugState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property(
            "ui_visible",
            &mut self.ui_visible,
            false,
        ));
        f(&mut cvar::Property(
            "render_visible",
            &mut self.render_visible,
            false,
        ));
        f(&mut cvar::List("render", &mut self.render));
        f(&mut cvar::ReadOnlyProp("fps", &mut self.fps, 0));
    }
}

pub fn build_ui(resources: &Resources, seconds_since_startup: u32, overstep_percentage: f32) {
    let game = resources.get::<MainState>().unwrap();
    let mut config = resources.get_mut::<Config>().unwrap();

    if mq::is_key_pressed(mq::KeyCode::GraveAccent) && mq::is_key_down(mq::KeyCode::LeftControl) {
        config.debug.ui_visible = !config.debug.ui_visible;
    }

    let (mouse_x, mouse_y) = mq::mouse_position();
    let game_x = mouse_x * GAME_WIDTH / WINDOW_WIDTH as f32;
    let game_y = mouse_y * GAME_HEIGHT / WINDOW_HEIGHT as f32;

    if config.debug.fps_second != seconds_since_startup {
        config.debug.fps = config.debug.fps_count;
        config.debug.fps_second = seconds_since_startup;
        config.debug.fps_count = 0;
    }
    config.debug.fps_count += 1;

    if config.debug.ui_visible {
        widgets::Window::new(hash!(), vec2(10., 10.), vec2(296., 91.))
            .titlebar(false)
            .ui(&mut *root_ui(), |ui| {
                if ui.button(
                    None,
                    toggle_button_label(config.debug.spawner_visible, "Spawn").as_str(),
                ) {
                    config.debug.spawner_visible = !config.debug.spawner_visible;
                }
                if ui.button(
                    Vec2::new(60., 2.),
                    toggle_button_label(config.debug.entities_visible, "Entities").as_str(),
                ) {
                    config.debug.entities_visible = !config.debug.entities_visible;
                }
                if ui.button(
                    Vec2::new(139., 2.),
                    toggle_button_label(config.debug.render_visible, "Render").as_str(),
                ) {
                    config.debug.render_visible = !config.debug.render_visible;
                }

                ui.separator();
                ui.label(
                    None,
                    format!("{:4}FPS \u{B1}{}", config.debug.fps, overstep_percentage).as_str(),
                );

                ui.label(
                    None,
                    format!(
                        " \u{86} {:4} {:3} [{:.3} {:.3}]",
                        mouse_x as u32, mouse_y as u32, game_x, game_y
                    )
                    .as_str(),
                );
                if ui.button(Vec2::new(6., 47.), "\u{86}") {
                    let mq::InternalGlContext {
                        quad_context: ctx, ..
                    } = unsafe { mq::get_internal_gl() };
                    ctx.set_cursor_grab(false);
                }
                let (dx, dy, _w, _h) = game.viewport(1.0);
                let (x, y) = game.mouse_to_world(1.0, game_x, game_y);
                ui.label(
                    None,
                    format!(" \u{AC} {:4.3} {:3.3} ({:.3},{:.3})", x, y, dx, dy).as_str(),
                );
            });

        // spawner::build_ui();
        render::build_ui(&mut config.debug);
    }

    // entities::build_ui();
}

pub(crate) fn toggle_button_label<S: std::fmt::Display>(state: bool, label: S) -> String {
    if state {
        format!("[{}]", label)
    } else {
        format!(" {} ", label)
    }
}

pub(crate) fn checkbox_label<S: std::fmt::Display>(state: bool, label: S) -> String {
    if state {
        format!("[x] {}", label)
    } else {
        format!("[ ] {}", label)
    }
}
