// https://github.com/Bombfuse/emerald/blob/master/src/core/engine.rs
use crate::mq;
use ringbuffer::{AllocRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};

mod frame_timer;
pub mod input;
mod miniquad;
mod script;
pub mod utils;
pub mod world;

use input::InputEngine;
use script::ScriptEngine;

use frame_timer::DESIRED_FRAMETIME;
const TIME_HISTORY_COUNT: usize = 4;

pub struct Engine<'a> {
    pub delta: f64,
    pub fps: usize,
    pub overstep_percentage: f32,
    pub quad_ctx: &'a mut mq::Context,
    pub egui_ctx: &'a egui::CtxRef,
    pub mouse_over_ui: bool,
    pub input: &'a mut InputEngine,
    pub script: &'a mut ScriptEngine,
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
    time_averager: AllocRingBuffer<f64>,
    frame_accumulator: f64,

    // renderer
    overstep_percentage: f32,
    last_frame: f64,
    fps_second: f64,
    fps_count: usize,
    fps: AllocRingBuffer<usize>,

    // engines
    pub(crate) input: InputEngine,
    pub(crate) script: ScriptEngine,

    // dependencies
    egui_mq: egui_miniquad::EguiMq,
    mouse_over_ui: bool,
}

impl<G: Game> Runner<G> {
    pub fn new(ctx: &mut mq::Context, mut game: G) -> Self {
        let mut time_averager = AllocRingBuffer::with_capacity(TIME_HISTORY_COUNT);
        time_averager.fill(DESIRED_FRAMETIME);

        let egui_mq = egui_miniquad::EguiMq::new(ctx);
        let mut input = InputEngine::new();
        let mut script = ScriptEngine::new();

        let eng = Engine {
            delta: 0.,
            fps: 0,
            overstep_percentage: 0.,
            quad_ctx: ctx,
            egui_ctx: egui_mq.egui_ctx(),
            mouse_over_ui: false,
            input: &mut input,
            script: &mut script,
        };
        game.initialize(eng);

        Runner {
            game,

            // frame_timer
            resync: true,
            prev_frame_time: mq::date::now(),
            time_averager,
            frame_accumulator: 0.0,
            overstep_percentage: 1.0,

            last_frame: mq::date::now(),
            fps_second: mq::date::now().round(),
            fps_count: 0,
            fps: AllocRingBuffer::with_capacity(64),

            input,
            script,

            egui_mq,
            mouse_over_ui: false,
        }
    }

    pub fn fps(&self) -> usize {
        *self.fps.get(-1).unwrap_or(&0)
    }
}
