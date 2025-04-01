// Based on https://github.com/Bombfuse/emerald/blob/master/src/core/engine.rs
use crate::mq;
use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub mod events;
mod frame_timer;
pub mod input;
mod logger;
mod miniquad;
mod script;
pub mod world;

pub use events::Event;
use input::{Direction, InputEngine, KeyBind, KeyMods};
pub use logger::Logger;
use script::ScriptEngine;

use frame_timer::DESIRED_FRAMETIME;
const TIME_HISTORY_COUNT: usize = 4;

pub struct Engine<'a> {
    pub now: f64,
    pub delta: f64,
    pub fps: usize,
    pub overstep_percentage: f32,
    pub mouse_over_ui: bool,
    pub input: &'a mut InputEngine,
    pub script: &'a mut ScriptEngine,
    pub event_sender: &'a BroadcastSender<Event>,
}

pub trait Game {
    fn initialize(&mut self, _quad_ctx: &mut mq::Context, _eng: Engine<'_>) {}
    fn update(&mut self, _eng: Engine<'_>) {}
    fn draw(&mut self, _quad_ctx: &mut mq::Context, _eng: Engine<'_>) {}
    fn draw_debug(&mut self, _egui_ctx: &egui::Context, _eng: Engine<'_>) {}
}

pub struct Runner<G: Game> {
    ctx: Box<mq::Context>,

    game: G,

    // frame_timer
    resync: bool,
    frame_time: f64,
    time_averager: AllocRingBuffer<f64>,
    frame_accumulator: f64,

    // renderer
    overstep_percentage: f32,
    render_time: f64,
    fps_second: f64,
    fps_count: usize,
    fps: AllocRingBuffer<usize>,

    // engines
    pub(crate) input: InputEngine,
    pub(crate) script: ScriptEngine,

    // events queue
    event_sender: BroadcastSender<Event>,

    // dependencies
    egui_mq: egui_miniquad::EguiMq,
    mouse_over_ui: bool,
}

impl<G: Game> Runner<G> {
    pub fn new(mut ctx: Box<mq::Context>, mut game: G) -> Self {
        let mut time_averager = AllocRingBuffer::new(TIME_HISTORY_COUNT);
        time_averager.fill(DESIRED_FRAMETIME);

        let (event_sender, event_recv) = broadcast_queue(64);

        let egui_mq = egui_miniquad::EguiMq::new(&mut *ctx);
        let mut input = InputEngine::new();
        let mut script = ScriptEngine::new(event_sender.clone(), event_recv);

        let now = mq::date::now();

        let eng = Engine {
            now,
            delta: 0.,
            fps: 0,
            overstep_percentage: 0.,
            mouse_over_ui: false,
            input: &mut input,
            script: &mut script,
            event_sender: &event_sender,
        };
        game.initialize(&mut *ctx, eng);

        Runner {
            ctx,
            game,

            // frame_timer
            resync: true,
            frame_time: now,
            time_averager,
            frame_accumulator: 0.0,
            overstep_percentage: 1.0,

            render_time: now,
            fps_second: now.round(),
            fps_count: 0,
            fps: AllocRingBuffer::new(64),

            input,
            script,

            event_sender,

            egui_mq,
            mouse_over_ui: false,
        }
    }

    pub fn fps(&self) -> usize {
        *self.fps.back().unwrap_or(&0)
    }
}
