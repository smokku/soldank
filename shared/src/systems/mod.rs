mod debug;
pub use debug::*;
mod movement;
pub use movement::*;

#[derive(Debug)]
pub struct Time {
    pub time: f64,
    pub tick: u64,
    pub frame_percent: f64,
}
