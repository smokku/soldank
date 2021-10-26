use crate::{
    orb::{
        command::Command,
        fixed_timestepper::Stepper,
        world::{DisplayState, World as OrbWorld},
    },
    physics::{self as physics, PhysicsEngine},
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
pub struct GameWorld {
    world: World,
    physics: PhysicsEngine,
}

#[derive(Debug, Clone, SerBin, DeBin)]
pub enum NetCommand {
    Command(String),
}

#[derive(Debug, Default, Clone, SerBin, DeBin)]
pub struct NetSnapshot {}

impl OrbWorld for GameWorld {
    type ClientId = SocketAddr;
    type CommandType = NetCommand;
    type SnapshotType = NetSnapshot;
    type DisplayStateType = World;

    fn command_is_valid(command: &NetCommand, client_id: Self::ClientId) -> bool {
        // Only localhost has permission to spawn
        match command {
            NetCommand::Command(_) => {
                client_id.ip() == "127.0.0.1:0".parse::<SocketAddr>().unwrap().ip()
            }
            _ => true,
        }
    }

    fn apply_command(&mut self, command: &NetCommand) {
        match command {
            NetCommand::Command(cmd) => match cmd.as_str() {
                "spawn" => self.spawn_object(),
                cmd => panic!("Unhandled command {}", cmd),
            },
        }
    }

    fn apply_snapshot(&mut self, snapshot: NetSnapshot) {
        todo!()
    }

    fn snapshot(&self) -> NetSnapshot {
        todo!();
        NetSnapshot {}
    }

    fn display_state(&self) -> Self::DisplayStateType {
        self.world.clone()
    }
}

impl Command for NetCommand {}

impl Stepper for GameWorld {
    fn step(&mut self) {
        physics::attach_bodies_and_colliders(&mut self.world);
        // physics::create_joints_system();
        physics::finalize_collider_attach_to_bodies(
            &mut self.world,
            &mut self.physics.modification_tracker,
        );

        physics::step_world(
            &mut self.world,
            &self.physics.gravity,
            &self.physics.integration_parameters,
            &mut self.physics.physics_pipeline,
            &mut self.physics.modification_tracker,
            &mut self.physics.island_manager,
            &mut self.physics.broad_phase,
            &mut self.physics.narrow_phase,
            &mut self.physics.joint_set,
            &mut self.physics.ccd_solver,
            &(),
            &(),
        );

        physics::despawn_outliers(&mut self.world, 2500., 16.); // FIXME: use config.phys.scale
        physics::collect_removals(&mut self.world, &mut self.physics.modification_tracker);
        // physics::config_update(&resources);

        self.world.clear_trackers();
        // resources.get_mut::<AppEventsQueue>().unwrap().clear();
    }
}

impl DisplayState for World {
    fn from_interpolation(state1: &Self, state2: &Self, t: f64) -> Self {
        state2.clone()
    }
}

impl GameWorld {
    pub fn spawn_object(&mut self) {
        todo!()
    }
}
