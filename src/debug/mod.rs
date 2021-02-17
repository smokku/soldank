use super::*;
use megaui_macroquad::{
    draw_window,
    megaui::{hash, Vector2},
    WindowParams,
};

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

pub fn build_ui(
    state: &mut DebugState,
    game: &MainState,
    seconds_since_startup: u32,
    overstep_percentage: f32,
) {
    if mq::is_key_pressed(mq::KeyCode::GraveAccent) && mq::is_key_down(mq::KeyCode::LeftControl) {
        state.ui_visible = !state.ui_visible;
    }

    let (mouse_x, mouse_y) = mq::mouse_position();
    let game_x = mouse_x * GAME_WIDTH / WINDOW_WIDTH as f32;
    let game_y = mouse_y * GAME_HEIGHT / WINDOW_HEIGHT as f32;

    if state.fps_second != seconds_since_startup {
        state.fps = state.fps_count;
        state.fps_second = seconds_since_startup;
        state.fps_count = 0;
    }
    state.fps_count += 1;

    if state.ui_visible {
        draw_window(
            hash!(),
            vec2(10., 10.),
            vec2(296., 89.),
            WindowParams {
                titlebar: false,
                ..Default::default()
            },
            |ui| {
                if ui.button(
                    None,
                    toggle_button_label(state.spawner_visible, "Spawn").as_str(),
                ) {
                    state.spawner_visible = !state.spawner_visible;
                }
                if ui.button(
                    Vector2::new(60., 2.),
                    toggle_button_label(state.entities_visible, "Entities").as_str(),
                ) {
                    state.entities_visible = !state.entities_visible;
                }
                if ui.button(
                    Vector2::new(139., 2.),
                    toggle_button_label(state.render_visible, "Render").as_str(),
                ) {
                    state.render_visible = !state.render_visible;
                }

                ui.separator();
                ui.label(
                    None,
                    format!("{:4}FPS \u{B1}{}", state.fps, overstep_percentage).as_str(),
                );

                ui.label(
                    None,
                    format!(
                        " \u{86} {:4} {:3} [{:.3} {:.3}]",
                        mouse_x as u32, mouse_y as u32, game_x, game_y
                    )
                    .as_str(),
                );
                if ui.button(Vector2::new(6., 47.), "\u{86}") {
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
            },
        );

        // spawner::build_ui();
        render::build_ui(state);
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
