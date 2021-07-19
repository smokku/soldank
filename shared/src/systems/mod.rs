mod debug;
pub use debug::*;
mod movement;
pub use movement::*;
mod input;
pub use input::*;

#[derive(Debug)]
pub struct Time {
    pub time: std::time::Instant,
    pub tick: usize,
    pub frame_percent: f64,
}
