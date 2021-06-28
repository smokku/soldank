use cvar::{INode, IVisit};
pub use soldank_shared::cvars::*;

#[derive(Default)]
pub struct Config {
    pub phys: Physics,
    pub network: NetConfig,
}

impl IVisit for Config {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::List("phys", &mut self.phys));
        f(&mut cvar::List("net", &mut self.network));
    }
}
