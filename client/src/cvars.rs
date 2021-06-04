use crate::debug::DebugState;
use cvar::{INode, IVisit};
use soldank_shared::cvars::Physics;

#[derive(Default)]
pub struct Config {
    pub phys: Physics,
    pub debug: DebugState,
}

impl IVisit for Config {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::List("phys", &mut self.phys));
        f(&mut cvar::List("debug", &mut self.debug));
    }
}
