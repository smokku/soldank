pub mod components;
pub mod constants;
pub mod control;
pub mod cvars;
pub mod messages;
pub mod networking;
pub mod orb;
pub mod physics;
pub mod systems;

use hexdump::hexdump_iter;

pub use glam as math;

pub fn trace_dump_packet(data: &[u8]) {
    for (n, line) in hexdump_iter(data).enumerate() {
        log::trace!(" {:3} {}", n, line);
    }
}
