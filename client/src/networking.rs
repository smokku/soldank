use gfx2d::macroquad::{logging as log, prelude as mq};

use std::{net::SocketAddr, time::Duration};

use naia_client::{find_my_ip_address, ClientConfig, ClientEvent, NaiaClient};

use soldank_shared::{
    get_shared_config, manifest_load, shared_behavior, AuthEvent, ExampleActor, ExampleEvent,
    KeyCommand, PointActorColor,
};

const SERVER_PORT: u16 = 14191;
pub struct Networking {
    client: NaiaClient<ExampleEvent, ExampleActor>,
    pawn_key: Option<u16>,
    queued_command: Option<KeyCommand>,
}

impl Networking {
    pub fn new() -> Self {
        let server_ip_address = find_my_ip_address().expect("can't find ip address");
        let server_socket_address = SocketAddr::new(server_ip_address, SERVER_PORT);

        let mut client_config = ClientConfig::default();
        client_config.heartbeat_interval = Duration::from_secs(2);
        client_config.disconnection_timeout_duration = Duration::from_secs(5);

        let auth = ExampleEvent::AuthEvent(AuthEvent::new("charlie", "12345"));

        let client = NaiaClient::new(
            server_socket_address,
            manifest_load(),
            Some(client_config),
            get_shared_config(),
            Some(auth),
        );

        Networking {
            client,
            pawn_key: None,
            queued_command: None,
        }
    }

    pub fn update(&mut self) {
        // input
        let w = mq::is_key_down(mq::KeyCode::W);
        let s = mq::is_key_down(mq::KeyCode::S);
        let a = mq::is_key_down(mq::KeyCode::A);
        let d = mq::is_key_down(mq::KeyCode::D);

        if let Some(command) = &mut self.queued_command {
            if w {
                command.w.set(true);
            }
            if s {
                command.s.set(true);
            }
            if a {
                command.a.set(true);
            }
            if d {
                command.d.set(true);
            }
        } else {
            self.queued_command = Some(KeyCommand::new(w, s, a, d));
        }

        // update
        loop {
            if let Some(result) = self.client.receive() {
                match result {
                    Ok(event) => match event {
                        ClientEvent::Connection => {
                            log::debug!("Client connected to: {}", self.client.server_address());
                        }
                        ClientEvent::Disconnection => {
                            log::debug!(
                                "Client disconnected from: {}",
                                self.client.server_address()
                            );
                        }
                        ClientEvent::Tick => {
                            if let Some(pawn_key) = self.pawn_key {
                                if let Some(command) = self.queued_command.take() {
                                    self.client.send_command(pawn_key, &command);
                                }
                            }
                        }
                        ClientEvent::AssignPawn(local_key) => {
                            self.pawn_key = Some(local_key);
                            log::debug!("assign pawn");
                        }
                        ClientEvent::UnassignPawn(_) => {
                            self.pawn_key = None;
                            log::debug!("unassign pawn");
                        }
                        ClientEvent::Command(pawn_key, command_type) => match command_type {
                            ExampleEvent::KeyCommand(key_command) => {
                                if let Some(typed_actor) = self.client.get_pawn_mut(&pawn_key) {
                                    match typed_actor {
                                        ExampleActor::PointActor(actor) => {
                                            shared_behavior::process_command(&key_command, actor);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    Err(err) => {
                        log::error!("Client Error: {}", err);
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn render(&mut self) {
        let square_size = 32.0;

        if self.client.has_connection() {
            // draw actors
            for actor_key in self.client.actor_keys().unwrap() {
                if let Some(actor) = self.client.get_actor(&actor_key) {
                    match actor {
                        ExampleActor::PointActor(point_actor) => {
                            let color = match point_actor.as_ref().borrow().color.get() {
                                PointActorColor::Red => mq::RED,
                                PointActorColor::Blue => mq::BLUE,
                                PointActorColor::Yellow => mq::YELLOW,
                            };
                            mq::draw_rectangle(
                                f32::from(*(point_actor.as_ref().borrow().x.get())),
                                f32::from(*(point_actor.as_ref().borrow().y.get())),
                                square_size,
                                square_size,
                                color,
                            );
                        }
                    }
                }
            }

            // draw pawns
            for pawn_key in self.client.pawn_keys().unwrap() {
                if let Some(actor) = self.client.get_pawn(&pawn_key) {
                    match actor {
                        ExampleActor::PointActor(point_actor) => {
                            mq::draw_rectangle(
                                f32::from(*(point_actor.as_ref().borrow().x.get())),
                                f32::from(*(point_actor.as_ref().borrow().y.get())),
                                square_size,
                                square_size,
                                mq::WHITE,
                            );
                        }
                    }
                }
            }
        }
    }
}
