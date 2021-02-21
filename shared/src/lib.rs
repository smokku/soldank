extern crate log;
extern crate naia_derive;

use naia_shared::{LinkConditionerConfig, Manifest, SharedConfig};
use std::time::Duration;

mod actors;
pub mod behaviors;
mod events;

pub use actors::{
    point::{PointActor, PointActorColor},
    NetworkActor,
};
pub use events::{auth::AuthEvent, key::KeyCommand, NetworkEvent};

pub fn get_manifest() -> Manifest<NetworkEvent, NetworkActor> {
    let mut manifest = Manifest::<NetworkEvent, NetworkActor>::new();

    manifest.register_event(AuthEvent::get_builder());
    manifest.register_pawn(PointActor::get_builder(), KeyCommand::get_builder());

    manifest
}

pub fn get_shared_config() -> SharedConfig {
    let tick_interval = Duration::from_millis(50);

    let link_condition = if cfg!(debug_assertions) {
        Some(LinkConditionerConfig::good_condition())
        // Some(LinkConditionerConfig {
        //     incoming_latency: 500,
        //     incoming_jitter: 1,
        //     incoming_loss: 0.0,
        //     incoming_corruption: 0.0,
        // })
    } else {
        None
    };

    return SharedConfig::new(tick_interval, link_condition);
}
