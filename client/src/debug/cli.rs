use super::*;
use macroquad::prelude::*;

#[derive(Default)]
pub struct CliState {
    pub(crate) visible: bool,

    history: Vec<String>,
    input: String,
    auto_scroll: Option<f32>,
}

impl IVisit for CliState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
    }
}

impl CliState {
    pub fn build_ui(self: &mut Self) {
        if self.visible {
            widgets::Window::new(hash!(), vec2(10., 110.), vec2(600., 280.))
                .label("Command Line Interface")
                .ui(&mut *root_ui(), |ui| {
                    for line in &self.history {
                        widgets::Label::new(line).ui(ui);
                    }
                    let input_id = hash!();
                    ui.input_text(input_id, "", &mut self.input);
                    ui.set_input_focus(input_id);

                    if let Some(last_y) = self.auto_scroll {
                        if last_y < ui.scroll_max().y {
                            ui.scroll_here_ratio(0.0);
                            self.auto_scroll = Some(-ui.scroll().y);
                        } else {
                            self.auto_scroll = None;
                        }
                    }

                    if ui.active_window_focused() && is_key_pressed(KeyCode::Enter) {
                        let input = self.input.trim();
                        if input.len() > 0 {
                            self.history.push(input.to_owned());
                            self.input.clear();
                            self.auto_scroll = Some(ui.scroll_max().y);
                        }
                    }
                });
        }
    }
}
