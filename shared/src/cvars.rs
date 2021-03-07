use cvar::{INode, IVisit};

const GRAV: f32 = 0.06;

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
