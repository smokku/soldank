// https://github.com/Bombfuse/emerald/blob/master/src/core/engine.rs
use crate::mq;
use std::collections::VecDeque;

mod frame_timer;
mod miniquad;

use frame_timer::DESIRED_FRAMETIME;
const TIME_HISTORY_COUNT: usize = 4;

pub struct Engine<'a> {
    delta: f64,
    // fps: f64,
    quad_ctx: &'a mut mq::Context,
    egui_ctx: &'a egui::CtxRef,
    mouse_over_ui: bool,
}

pub trait Game {
    fn initialize(&mut self, _eng: &mut Engine<'_>) {}
    fn update(&mut self, _eng: &mut Engine<'_>) {}
    fn draw(&mut self, mut _eng: &mut Engine<'_>) {}
}

pub struct Runner<G: Game> {
    game: G,

    // frame_timer
    resync: bool,
    prev_frame_time: f64,
    time_averager: VecDeque<f64>,
    frame_accumulator: f64,
    frame_percentage: f64,

    // dependencies
    egui_mq: egui_miniquad::EguiMq,
    mouse_over_ui: bool,
}

impl<G: Game> Runner<G> {
    pub fn new(ctx: &mut mq::Context, game: G) -> Self {
        let mut time_averager = VecDeque::with_capacity(TIME_HISTORY_COUNT);
        time_averager.resize(TIME_HISTORY_COUNT, DESIRED_FRAMETIME);

        Runner {
            game,

            // frame_timer
            resync: true,
            prev_frame_time: mq::date::now(),
            time_averager,
            frame_accumulator: 0.0,
            frame_percentage: 1.0,

            egui_mq: egui_miniquad::EguiMq::new(ctx),
            mouse_over_ui: false,
        }
    }
}
