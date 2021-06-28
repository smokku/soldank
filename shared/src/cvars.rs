use cvar::{INode, IVisit};

use crate::constants::*;

pub fn set_cli_cvars(config: &mut dyn IVisit, cmd: &clap::ArgMatches) {
    if let Some(values) = cmd.values_of("set") {
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

    log::info!("--- cvars:");
    cvar::console::walk(config, |path, node| match node.as_node() {
        cvar::Node::Prop(prop) => {
            log::info!("{} = `{}`", path, prop.get());
        }
        _ => {}
    });
}

pub struct Physics {
    pub gravity: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self { gravity: GRAV }
    }
}

impl IVisit for Physics {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("gravity", &mut self.gravity, GRAV));
    }
}

pub struct NetConfig {
    ticks_per_second: u8,
    snapshots_per_second: u8,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            ticks_per_second: TICKS_PER_SECOND,
            snapshots_per_second: SNAPSHOT_PER_SECOND,
        }
    }
}

impl IVisit for NetConfig {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property(
            "ticks_per_second",
            &mut self.ticks_per_second,
            TICKS_PER_SECOND,
        ));
        f(&mut cvar::Property(
            "snapshots_per_second",
            &mut self.snapshots_per_second,
            SNAPSHOT_PER_SECOND,
        ));
    }
}
