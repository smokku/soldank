use crate::{
    orb::{
        command::Command,
        fixed_timestepper::Stepper,
        world::{DisplayState, World as OrbWorld},
    },
    world::World,
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
pub struct NetWorld {
    world: World,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum NetCommand {
    Spawn,
}

#[derive(Debug, Default, Clone, SerBin, DeBin)]
pub struct NetSnapshot {}

impl OrbWorld for NetWorld {
    type ClientId = SocketAddr;
    type CommandType = NetCommand;
    type SnapshotType = NetSnapshot;
    type DisplayStateType = World;

    fn command_is_valid(command: &NetCommand, client_id: Self::ClientId) -> bool {
        // Only localhost has permission to spawn
        match command {
            NetCommand::Spawn => {
                client_id.ip() == "127.0.0.1:0".parse::<SocketAddr>().unwrap().ip()
            }
            _ => true,
        }
    }

    fn apply_command(&mut self, command: &NetCommand) {
        match command {
            NetCommand::Spawn => self.spawn_object(),
        }
    }

    fn apply_snapshot(&mut self, snapshot: NetSnapshot) {}

    fn snapshot(&self) -> NetSnapshot {
        NetSnapshot {}
    }

    fn display_state(&self) -> Self::DisplayStateType {
        self.world.clone()
    }
}

impl Command for NetCommand {}

impl Stepper for NetWorld {
    fn step(&mut self) {}
}

impl DisplayState for World {
    fn from_interpolation(state1: &Self, state2: &Self, t: f64) -> Self {
        state2.clone()
    }
}

impl NetWorld {
    pub fn spawn_object(&mut self) {}
}
