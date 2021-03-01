pub mod constants;
pub mod control;
pub mod messages;

use hexdump::hexdump_iter;

pub fn trace_dump_packet(data: &[u8]) {
    for (n, line) in hexdump_iter(data).enumerate() {
        log::trace!(" {:3} {}", n, line);
    }
}
