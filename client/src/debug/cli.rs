use super::*;
use crate::engine::{Event, Logger};

pub struct CliState {
    pub(crate) visible: bool,

    input: String,

    pub(crate) auto_focus: bool,
    pub(crate) auto_scroll: bool,
}

impl Default for CliState {
    fn default() -> Self {
        Self {
            visible: false,
            input: Default::default(),
            auto_focus: true,
            auto_scroll: true,
        }
    }
}

impl IVisit for CliState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
    }
}

impl CliState {
    pub fn build_ui(&mut self, eng: &Engine<'_>) {
        if !self.visible {
            return;
        }

        let egui_ctx = eng.egui_ctx;
        let mut visible = self.visible;
        egui::Window::new("Command Line Interface")
            .open(&mut visible)
            .resizable(true)
            .show(egui_ctx, |ui| {
                let scroll_height = ui.fonts()[egui::TextStyle::Monospace].row_height() * 25.;
                egui::ScrollArea::from_max_height(scroll_height)
                    // .always_show_scroll(true)
                    .show(ui, |ui| {
                        for (level, _time, line) in Logger::get_log().read().unwrap().iter() {
                            ui.add(
                                egui::Label::new(line)
                                    .text_style(egui::TextStyle::Monospace)
                                    .text_color(match level {
                                        log::Level::Info => egui::Color32::WHITE,
                                        log::Level::Warn => egui::Color32::YELLOW,
                                        log::Level::Error => egui::Color32::RED,
                                        log::Level::Debug | log::Level::Trace => {
                                            // should not really happen
                                            egui::Color32::BLACK
                                        }
                                    }),
                            );
                        }

                        if self.auto_scroll {
                            self.auto_scroll = false;
                            ui.scroll_to_cursor(egui::Align::BOTTOM);
                        }
                    });

                let edit = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .code_editor()
                        .desired_width(f32::INFINITY),
                );
                if self.auto_focus {
                    self.auto_focus = false;
                    edit.request_focus();
                }

                if edit.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    let input = self.input.trim();
                    if !input.is_empty() {
                        if let Err(err) =
                            eng.event_sender.try_send(Event::Command(input.to_string()))
                        {
                            log::error!("Cannot send Command Event: {}", err);
                        }
                        self.input.clear();
                        self.auto_scroll = true;
                        self.auto_focus = true;
                    }
                }
            });

        self.visible = visible;
    }
}
