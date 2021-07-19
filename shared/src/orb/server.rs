//! This module houses the structures that are specific to the management of a CrystalOrb game
//! server.
//!
//! The [`Server`] structure is what you create, store, and update, analogous to the
//! [`Client`](crate::client::Client) structure. Unlike the [`Client`](crate::client::Client), the
//! [`Server`] does not need a "loading" stage, and can be used directly after creation.
//!
//! The interface shares some similarities with the the [`Ready`](crate::client::stage::Ready)
//! client stage.

use super::{
    fixed_timestepper::{FixedTimestepper, TerminationCondition, TimeKeeper},
    // network_resource::{Connection, ConnectionHandleType, NetworkResource},
    timestamp::{Timestamp, Timestamped},
    world::{InitializationType, Simulation, World},
    Config,
};

/// This is the top-level structure of CrystalOrb for your game server, analogous to the
/// [`Client`](crate::client::Client) for game clients. You create, store, and update this server
/// instance to run your game on the server side.
#[derive(Debug)]
pub struct Server<'a, WorldType: World> {
    timekeeping_simulation: TimeKeeper<
        'a,
        Simulation<WorldType /*, { InitializationType::PreInitialized }*/>,
        // { TerminationCondition::LastUndershoot },
    >,
    seconds_since_last_snapshot: f64, // FIXME: "seconds since" is not constant
    config: &'a Config,

    incoming_commands: Vec<(Timestamped<WorldType::CommandType>, WorldType::ClientId)>,
    outgoing_commands: Vec<(
        Option<WorldType::ClientId>, // From (None = internal)
        Option<WorldType::ClientId>, // To   (None = broadcast)
        Timestamped<WorldType::CommandType>,
    )>,
    outgoing_snapshots: Vec<Timestamped<WorldType::SnapshotType>>,
}

impl<'a, WorldType: World> Server<'a, WorldType> {
    /// Constructs a new [`Server`]. This function requires a `seconds_since_startup` parameter to
    /// initialize the server's simulation timestamp.
    pub fn new(config: &'a Config, seconds_since_startup: f64) -> Self {
        let mut server = Self {
            timekeeping_simulation: TimeKeeper::new(
                Simulation::new(InitializationType::PreInitialized),
                config,
                TerminationCondition::LastUndershoot,
            ),
            seconds_since_last_snapshot: 0.0,
            config,
            incoming_commands: Vec::new(),
            outgoing_commands: Vec::new(),
            outgoing_snapshots: Vec::new(),
        };

        let initial_timestamp =
            Timestamp::from_seconds(seconds_since_startup, server.config.timestep_seconds)
                - server.config.lag_compensation_frame_count();
        server
            .timekeeping_simulation
            .reset_last_completed_timestamp(initial_timestamp);

        server
    }

    /// The timestamp of the most recent frame that has completed its simulation.
    /// This is typically one less than [`Server::simulating_timestamp`].
    pub fn last_completed_timestamp(&self) -> Timestamp {
        self.timekeeping_simulation.last_completed_timestamp()
    }

    /// The timestamp of the frame that is *in the process* of being simulated.
    /// This is typically one more than [`Server::simulating_timestamp`].
    pub fn simulating_timestamp(&self) -> Timestamp {
        self.timekeeping_simulation.simulating_timestamp()
    }

    /// The timestamp that clients are supposed to be simulating at the moment (which should always
    /// be ahead of the server to compensate for the latency between the server and the clients).
    ///
    /// This is also the timestamp that gets attached to the command when you call
    /// [`Server::issue_command`].
    pub fn estimated_client_simulating_timestamp(&self) -> Timestamp {
        self.simulating_timestamp() + self.config.lag_compensation_frame_count()
    }

    /// The timestamp that clients have supposed to have completed simulating (which should always
    /// be ahead of the server to compensate for the latency between the server and the clients).
    pub fn estimated_client_last_completed_timestamp(&self) -> Timestamp {
        self.last_completed_timestamp() + self.config.lag_compensation_frame_count()
    }

    fn apply_validated_command(
        &mut self,
        command: &Timestamped<WorldType::CommandType>,
        command_source: Option<WorldType::ClientId>,
        // net: &mut NetworkResourceType,
    ) {
        log::debug!("Received command from {:?} - {:?}", command_source, command);

        // Apply this command to our world later on.
        self.timekeeping_simulation.schedule_command(command);

        // Relay command to every other client.
        // for (handle, mut connection) in net.connections() {
        //     // Don't send it back to the same client if there is one.
        //     if let Some(source_handle) = command_source {
        //         if handle == source_handle {
        //             continue;
        //         }
        //     }
        //     let result = connection.send(command.clone());
        //     connection.flush::<Timestamped<WorldType::CommandType>>();
        //     if let Some(message) = result {
        //         log::error!("Failed to relay command to [{}]: {:?}", handle, message);
        //     }
        // }
        self.outgoing_commands
            .push((command_source, None, command.clone()));
    }

    fn receive_command(
        &mut self,
        command: &Timestamped<WorldType::CommandType>,
        command_source: WorldType::ClientId,
        // net: &mut NetworkResourceType,
    ) {
        if WorldType::command_is_valid(command.inner(), command_source)
        // TODO: Is it valid to validate the timestamps?
        // && command.timestamp() >= self.timekeeping_simulation.last_completed_timestamp()
        // && command.timestamp() <= self.estimated_client_simulating_timestamp()
        {
            self.apply_validated_command(&command, Some(command_source));
        }
    }

    /// Issue a command from the server to the world. The command will be scheduled to the
    /// estimated client's current timestamp.
    pub fn issue_command(
        &mut self,
        command: WorldType::CommandType,
        // net: &mut NetworkResourceType,
    ) {
        self.apply_validated_command(
            &Timestamped::new(command, self.estimated_client_simulating_timestamp()),
            None,
            // net,
        );
    }

    /// Iterate through the commands that are being kept around. This is intended to be for
    /// diagnostic purposes.
    pub fn buffered_commands(
        &self,
    ) -> impl Iterator<Item = (Timestamp, &Vec<WorldType::CommandType>)> {
        self.timekeeping_simulation.buffered_commands()
    }

    /// Get the current display state of the server's world.
    pub fn display_state(&self) -> Timestamped<WorldType::DisplayStateType> {
        self.timekeeping_simulation
            .display_state()
            .expect("Server simulation does not need initialization")
    }

    /// Perform the next update. You would typically call this in your game engine's update loop of
    /// some kind.
    pub fn update(
        &mut self,
        delta_seconds: f64,
        seconds_since_startup: f64,
        // net: &mut NetworkResourceType,
    ) {
        let positive_delta_seconds = delta_seconds.max(0.0);
        #[allow(clippy::float_cmp)]
        if delta_seconds != positive_delta_seconds {
            log::warn!(
                "Attempted to update client with a negative delta_seconds of {}. Clamping it to zero.",
                delta_seconds
            );
        }
        // let mut new_commands = Vec::new();
        // let mut clock_syncs = Vec::new();
        // for (handle, mut connection) in net.connections() {
        //     while let Some(command) = connection.recv_command() {
        //         new_commands.push((command, handle));
        //     }
        //     while let Some(mut clock_sync_message) = connection.recv_clock_sync() {
        //         log::trace!("Replying to clock sync message. client_id: {}", handle);
        //         clock_sync_message.server_seconds_since_startup = seconds_since_startup;
        //         clock_sync_message.client_id = handle;
        //         clock_syncs.push((handle, clock_sync_message));
        //     }
        // }
        let commands = self.incoming_commands.split_off(0);
        for (command, command_source) in &commands {
            self.receive_command(command, *command_source);
        }
        // for (handle, clock_sync_message) in clock_syncs {
        //     net.send_message(handle, clock_sync_message)
        //         .expect("Connection from which clocksync request came from should still exist");
        // }

        self.timekeeping_simulation
            .update(positive_delta_seconds, seconds_since_startup);

        self.seconds_since_last_snapshot += positive_delta_seconds;
        if self.seconds_since_last_snapshot > self.config.snapshot_send_period {
            log::trace!(
                "Broadcasting snapshot at timestamp: {:?} (note: drift error: {})",
                self.timekeeping_simulation.last_completed_timestamp(),
                self.timekeeping_simulation
                    .timestamp_drift_seconds(seconds_since_startup),
            );
            self.seconds_since_last_snapshot = 0.0;
            // net.broadcast_message(self.timekeeping_simulation.last_completed_snapshot());
            self.outgoing_snapshots
                .push(self.timekeeping_simulation.last_completed_snapshot());
        }
    }

    /// How these message channels get multiplexed is up to you / the external networking library.
    /// Whether or not these message channels are reliable or unreliable, ordered or unordered, is also
    /// up to you / the external networking library. CrystalOrb is written assuming that
    /// `ClockSyncMessage` and `SnapshotType` are unreliable and unordered, while `CommandType` is
    /// reliable but unordered.
    pub fn take_outgoing_snapshots(&mut self) -> Vec<Timestamped<WorldType::SnapshotType>> {
        self.outgoing_snapshots.split_off(0)
    }

    /// CrystalOrb is written assuming that
    /// `ClockSyncMessage` and `SnapshotType` are unreliable and unordered, while `CommandType` is
    /// reliable but unordered.
    pub fn take_outgoing_commands(
        &mut self,
    ) -> Vec<(
        Option<WorldType::ClientId>, // From (None = internal)
        Option<WorldType::ClientId>, // To   (None = broadcast)
        Timestamped<WorldType::CommandType>,
    )> {
        self.outgoing_commands.split_off(0)
    }

    pub fn enqueue_incoming_command(
        &mut self,
        command: Timestamped<WorldType::CommandType>,
        from: WorldType::ClientId,
    ) {
        self.incoming_commands.push((command, from));
    }
}
