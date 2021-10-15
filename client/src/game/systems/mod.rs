use crate::{
    engine::{input::InputState, Engine},
    game::components::{Input, Pawn},
    math::*,
    particles::Particle,
    render::components::*,
};
use hecs::{With, World};

mod debug;
mod movement;
pub use debug::*;
pub use movement::*;

pub fn apply_input(world: &mut World, eng: &Engine) {
    for (_, mut input) in world.query::<With<Pawn, &mut Input>>().iter() {
        input.state = eng.input.state;
    }
}
