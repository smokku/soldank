use crate::constants::*;
use cvar::{INode, IVisit};
use std::sync::{Arc, RwLock};

pub fn set_cli_cvars(config: &mut dyn IVisit, cmd: &clap::ArgMatches) {
    if let Some(values) = cmd.get_many::<String>("set") {
        for chunk in values.collect::<Vec<_>>().chunks_exact(2) {
            let cvar = chunk[0];
            let value = chunk[1];
            match cvar::console::set(config, cvar, value) {
                Ok(set) => {
                    if !set {
                        log::error!(
                            "Cannot set cvar `{} = {}`: cvar not available.",
                            cvar,
                            value
                        );
                    }
                }
                Err(err) => {
                    log::error!("Cannot parse `{} = {}`: {}.", cvar, value, err);
                }
            }
        }
    }
}

pub fn dump_cvars(config: &mut dyn IVisit) {
    log::info!("--- cvars:");
    cvar::console::walk(config, |path, node| {
        if let cvar::Node::Prop(prop) = node.as_node() {
            log::info!("{} = `{}`", path, prop.get());
        }
    });
}

pub struct Physics {
    pub scale: f32,
    pub gravity: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            scale: PHYSICS_SCALE,
            gravity: GRAV,
        }
    }
}

impl IVisit for Physics {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("scale", &mut self.scale, PHYSICS_SCALE));
        f(&mut cvar::Property("gravity", &mut self.gravity, GRAV));
    }
}

#[derive(Default)]
pub struct NetConfig {
    pub send_keepalive: u32,    // millis
    pub keepalive_timeout: u32, // millis
}

impl IVisit for NetConfig {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property(
            "send_keepalive",
            &mut self.send_keepalive,
            0,
        ));
        f(&mut cvar::Property(
            "keepalive_timeout",
            &mut self.keepalive_timeout,
            0,
        ));
        // self.orb.write().unwrap().visit(f);
    }
}

// impl IVisit for OrbConfig {
//     fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
//         let default = Self::default();
//         f(&mut cvar::Property(
//             "lag_compensation_latency",
//             &mut self.lag_compensation_latency,
//             default.lag_compensation_latency,
//         ));
//         f(&mut cvar::Property(
//             "blend_latency",
//             &mut self.blend_latency,
//             default.blend_latency,
//         ));
//         f(&mut cvar::Property(
//             "timestep_seconds",
//             &mut self.timestep_seconds,
//             default.timestep_seconds,
//         ));
//         f(&mut cvar::Property(
//             "clock_sync_needed_sample_count",
//             &mut self.clock_sync_needed_sample_count,
//             default.clock_sync_needed_sample_count,
//         ));
//         f(&mut cvar::Property(
//             "clock_sync_assumed_outlier_rate",
//             &mut self.clock_sync_assumed_outlier_rate,
//             default.clock_sync_assumed_outlier_rate,
//         ));
//         f(&mut cvar::Property(
//             "clock_sync_request_period",
//             &mut self.clock_sync_request_period,
//             default.clock_sync_request_period,
//         ));
//         f(&mut cvar::Property(
//             "max_tolerable_clock_deviation",
//             &mut self.max_tolerable_clock_deviation,
//             default.max_tolerable_clock_deviation,
//         ));
//         f(&mut cvar::Property(
//             "snapshot_send_period",
//             &mut self.snapshot_send_period,
//             default.snapshot_send_period,
//         ));
//         f(&mut cvar::Property(
//             "update_delta_seconds_max",
//             &mut self.update_delta_seconds_max,
//             default.update_delta_seconds_max,
//         ));
//         f(&mut cvar::Property(
//             "timestamp_skip_threshold_seconds",
//             &mut self.timestamp_skip_threshold_seconds,
//             default.timestamp_skip_threshold_seconds,
//         ));
//         f(&mut cvar::Property(
//             "fastforward_max_per_step",
//             &mut self.fastforward_max_per_step,
//             default.fastforward_max_per_step,
//         ));
//         f(&mut cvar::Property(
//             "tweening_method",
//             &mut self.tweening_method,
//             default.tweening_method,
//         ));
//     }
// }

// impl INode for orb::TweeningMethod {
//     fn name(&self) -> &str {
//         todo!()
//     }

//     fn as_node(&mut self) -> cvar::Node<'_> {
//         todo!()
//     }

//     fn as_inode(&mut self) -> &mut dyn INode {
//         todo!()
//     }
// }
