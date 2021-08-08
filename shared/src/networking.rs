use crate::orb::{
    command::Command,
    fixed_timestepper::Stepper,
    world::{DisplayState, World},
};
use nanoserde::{DeBin, SerBin};
use std::{fmt::Debug, net::SocketAddr, time::Instant};

#[derive(Debug, Clone)]
pub struct PacketStats {
    pub packets_tx: usize,
    pub packets_rx: usize,
    pub bytes_tx: usize,
    pub bytes_rx: usize,
    pub last_tx: Instant,
    pub last_rx: Instant,
}

impl Default for PacketStats {
    fn default() -> Self {
        Self {
            packets_tx: Default::default(),
            packets_rx: Default::default(),
            bytes_tx: Default::default(),
            bytes_rx: Default::default(),
            last_tx: Instant::now(),
            last_rx: Instant::now(),
        }
    }
}

impl PacketStats {
    pub fn add_tx(&mut self, num_bytes: usize) {
        self.last_tx = Instant::now();
        self.packets_tx += 1;
        self.bytes_tx += num_bytes;
    }
    pub fn add_rx(&mut self, num_bytes: usize) {
        self.last_rx = Instant::now();
        self.packets_rx += 1;
        self.bytes_rx += num_bytes;
    }
}

#[derive(Default)]
pub struct MyWorld {
    position: f64,
    velocity: f64,

    // Your World implementation might contain cached state/calculations, for example.
    cached_momentum: Option<f64>,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum MyCommand {
    // Here, you would put down the things that you want to externally affect the physics
    // simulation. The most common would be player commands. Other things might include spawning
    // npcs or triggering high-level events if they are not part of the physics simulation.
    Accelerate,
    Decelerate,
    Cheat,
}

#[derive(Debug, Default, Clone, SerBin, DeBin)]
pub struct MySnapshot {
    // Here, you would probably want to put down the minimal subset of states that can be used to
    // describe the whole physics simulation at any point of time.
    position: f64,
    velocity: f64,
}

#[derive(Clone, Default, Debug)]
pub struct MyDisplayState {
    position: f64,
    // Unless you use the velocity value for rendering in some way (e.g. motion blur), you might
    // not need to include it here in this display state.
    velocity: f64,
    // You might also include other derived state that are useful for rendering.
}

impl World for MyWorld {
    type ClientId = SocketAddr;
    type CommandType = MyCommand;
    type SnapshotType = MySnapshot;
    type DisplayStateType = MyDisplayState;

    fn command_is_valid(command: &MyCommand, client_id: Self::ClientId) -> bool {
        // Only localhost has permission to cheat, for example.
        match command {
            MyCommand::Cheat => client_id.ip() == "127.0.0.1:0".parse::<SocketAddr>().unwrap().ip(),
            _ => true,
        }
    }

    fn apply_command(&mut self, command: &MyCommand) {
        match command {
            MyCommand::Accelerate => self.velocity += 1.0,
            MyCommand::Decelerate => self.velocity -= 1.0,
            MyCommand::Cheat => self.position = 0.0,
        }
    }

    fn apply_snapshot(&mut self, snapshot: MySnapshot) {
        self.position = snapshot.position;
        self.velocity = snapshot.velocity;
        self.cached_momentum = None;
    }

    fn snapshot(&self) -> MySnapshot {
        MySnapshot {
            position: self.position,
            velocity: self.velocity,
        }
    }

    fn display_state(&self) -> MyDisplayState {
        MyDisplayState {
            position: self.position,
            velocity: self.velocity,
        }
    }
}

impl Command for MyCommand {}

impl Stepper for MyWorld {
    fn step(&mut self) {
        const DELTA_SECONDS: f64 = 1.0 / 60.0;
        const MASS: f64 = 2.0;
        self.position += self.velocity * DELTA_SECONDS;
        self.cached_momentum = Some(self.velocity * MASS);
    }
}

impl DisplayState for MyDisplayState {
    fn from_interpolation(state1: &Self, state2: &Self, t: f64) -> Self {
        MyDisplayState {
            position: (1.0 - t) * state1.position + t * state2.position,
            velocity: (1.0 - t) * state1.velocity + t * state2.velocity,
            // You can, for example, also do some more complex interpolation such as SLERP for
            // things that undergo rotation. To prevent some weird interpolation glitches (such as
            // deformable bodies imploding into themselves), you may need to transform points into
            // their local coordinates before interpolating.
        }
    }
}
