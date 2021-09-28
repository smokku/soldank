// https://github.com/Bombfuse/emerald/blob/master/src/core/engine.rs
use crate::mq;
use std::collections::VecDeque;

mod frame_timer;
mod input;
mod miniquad;

use input::InputEngine;

use frame_timer::DESIRED_FRAMETIME;
const TIME_HISTORY_COUNT: usize = 4;

pub struct Engine<'a> {
    pub delta: f64,
    pub fps: f32,
    pub overstep_percentage: f32,
    pub quad_ctx: &'a mut mq::Context,
    pub egui_ctx: &'a egui::CtxRef,
    pub mouse_over_ui: bool,
    pub input: &'a mut InputEngine,
}

pub trait Game {
    fn initialize(&mut self, _eng: Engine<'_>) {}
    fn update(&mut self, _eng: Engine<'_>) {}
    fn draw(&mut self, _eng: Engine<'_>) {}
}

pub struct Runner<G: Game> {
    game: G,

    // frame_timer
    resync: bool,
    prev_frame_time: f64,
    time_averager: VecDeque<f64>,
    frame_accumulator: f64,

    // renderer
    overstep_percentage: f32,
    fps: f32,

    // engines
    pub(crate) input: InputEngine,

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
            overstep_percentage: 1.0,

            fps: 0.0,

            input: InputEngine::new(),

            egui_mq: egui_miniquad::EguiMq::new(ctx),
            mouse_over_ui: false,
        }
    }
}

impl<'a> Engine<'a> {
    pub(crate) fn new<G: Game>(
        runner: &'a mut Runner<G>,
        delta: f64,
        quad_ctx: &'a mut mq::Context,
    ) -> Self {
        Engine {
            delta,
            fps: runner.fps,
            overstep_percentage: runner.overstep_percentage,
            quad_ctx,
            egui_ctx: runner.egui_mq.egui_ctx(),
            mouse_over_ui: runner.mouse_over_ui,
            input: &mut runner.input,
        }
    }
}
