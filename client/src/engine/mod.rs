// https://github.com/Bombfuse/emerald/blob/master/src/core/engine.rs
use crate::mq;
use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use ringbuffer::{AllocRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};

pub mod events;
mod frame_timer;
pub mod input;
mod logger;
mod miniquad;
mod script;
pub mod world;

pub use events::Event;
use input::{InputEngine, KeyBind, KeyMods};
pub use logger::Logger;
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
    pub event_sender: &'a BroadcastSender<Event>,
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

    // events queue
    event_sender: BroadcastSender<Event>,

    // dependencies
    egui_mq: egui_miniquad::EguiMq,
    mouse_over_ui: bool,
}

impl<G: Game> Runner<G> {
    pub fn new(ctx: &mut mq::Context, mut game: G) -> Self {
        let mut time_averager = AllocRingBuffer::with_capacity(TIME_HISTORY_COUNT);
        time_averager.fill(DESIRED_FRAMETIME);

        let (event_sender, event_recv) = broadcast_queue(64);
        // Take notice that I drop the receiver - this removes it from
        // the queue, meaning that other readers
        // won't get starved by the lack of progress here

        let egui_mq = egui_miniquad::EguiMq::new(ctx);
        let mut input = InputEngine::new();
        let mut script = ScriptEngine::new(event_sender.clone(), event_recv.clone());

        let eng = Engine {
            delta: 0.,
            fps: 0,
            overstep_percentage: 0.,
            quad_ctx: ctx,
            egui_ctx: egui_mq.egui_ctx(),
            mouse_over_ui: false,
            input: &mut input,
            script: &mut script,
            event_sender: &event_sender,
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

            event_sender,

            egui_mq,
            mouse_over_ui: false,
        }
    }

    pub fn fps(&self) -> usize {
        *self.fps.get(-1).unwrap_or(&0)
    }
}
