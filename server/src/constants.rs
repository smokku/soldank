use std::time::Duration;

pub use soldank_shared::constants::*;

pub const BROADCAST_RATE: f64 = 1.0 / 3.0;
pub const MAX_NETWORK_IDLE: Duration = Duration::from_millis(((1.0 / 10.0) * 1000.) as _);
