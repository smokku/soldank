use cvar::{INode, IVisit};
pub use soldank_shared::cvars::*;

#[derive(Default)]
pub struct Config {
    pub server: ServerInfo,
    pub net: NetConfig,
    pub phys: Physics,
}

impl IVisit for Config {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::List("server", &mut self.server));
        f(&mut cvar::List("net", &mut self.net));
        f(&mut cvar::List("phys", &mut self.phys));
    }
}

pub struct ServerInfo {
    pub motd: String,
}

fn default_motd() -> String {
    format!(
        "{} {} - {}",
        clap::crate_name!(),
        clap::crate_version!(),
        clap::crate_description!()
    )
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            motd: default_motd(),
        }
    }
}

impl IVisit for ServerInfo {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("motd", &mut self.motd, default_motd()));
    }
}
