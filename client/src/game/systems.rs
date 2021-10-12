use crate::{
    engine::Engine,
    game::components::{Input, Pawn},
};
use hecs::{With, World};

pub fn apply_input(world: &mut World, eng: &Engine) {
    for (_, mut input) in world.query::<With<Pawn, &mut Input>>().iter() {
        input.state = eng.input.state;
    }
}
