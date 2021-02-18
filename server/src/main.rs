#[macro_use]
extern crate log;

use simple_logger::SimpleLogger;
use smol::io;

use naia_server::{
    find_my_ip_address, ActorKey, NaiaServer, Random, ServerConfig, ServerEvent, UserKey,
};

use soldank_shared::{
    get_shared_config, manifest_load, shared_behavior, ExampleActor, ExampleEvent, PointActor,
    PointActorColor,
};

use std::{collections::HashMap, net::SocketAddr, rc::Rc, time::Duration};

const SERVER_PORT: u16 = 14191;

fn main() -> io::Result<()> {
    smol::block_on(async {
        SimpleLogger::from_env()
            .init()
            .expect("A logger was already initialized");

        info!("Soldank Server Started");

        let current_ip_address = find_my_ip_address().expect("can't find ip address");
        let current_socket_address = SocketAddr::new(current_ip_address, SERVER_PORT);

        let mut server_config = ServerConfig::default();
        server_config.heartbeat_interval = Duration::from_secs(2);
        server_config.disconnection_timeout_duration = Duration::from_secs(5);

        let mut server = NaiaServer::new(
            current_socket_address,
            manifest_load(),
            Some(server_config),
            get_shared_config(),
        )
        .await;

        server.on_auth(Rc::new(Box::new(|_, auth_type| {
            if let ExampleEvent::AuthEvent(auth_event) = auth_type {
                let username = auth_event.username.get();
                let password = auth_event.password.get();
                return username == "charlie" && password == "12345";
            }
            return false;
        })));

        let main_room_key = server.create_room();

        server.on_scope_actor(Rc::new(Box::new(|_, _, _, actor| match actor {
            ExampleActor::PointActor(_) => {
                return true;
            }
        })));

        let mut user_to_pawn_map = HashMap::<UserKey, ActorKey>::new();

        loop {
            match server.receive().await {
                Ok(event) => {
                    match event {
                        ServerEvent::Connection(user_key) => {
                            server.room_add_user(&main_room_key, &user_key);
                            if let Some(user) = server.get_user(&user_key) {
                                info!("Naia Server connected to: {}", user.address);

                                let x = Random::gen_range_u32(0, 50) * 16;
                                let y = Random::gen_range_u32(0, 37) * 16;

                                let actor_color = match server.get_users_count() % 3 {
                                    0 => PointActorColor::Yellow,
                                    1 => PointActorColor::Red,
                                    _ => PointActorColor::Blue,
                                };

                                let new_actor =
                                    PointActor::new(x as u16, y as u16, actor_color).wrap();
                                let new_actor_key = server
                                    .register_actor(ExampleActor::PointActor(new_actor.clone()));
                                server.room_add_actor(&main_room_key, &new_actor_key);
                                server.assign_pawn(&user_key, &new_actor_key);
                                user_to_pawn_map.insert(user_key, new_actor_key);
                            }
                        }
                        ServerEvent::Disconnection(user_key, user) => {
                            info!("Naia Server disconnected from: {:?}", user.address);
                            server.room_remove_user(&main_room_key, &user_key);
                            if let Some(actor_key) = user_to_pawn_map.remove(&user_key) {
                                server.room_remove_actor(&main_room_key, &actor_key);
                                server.unassign_pawn(&user_key, &actor_key);
                                server.deregister_actor(actor_key);
                            }
                        }
                        ServerEvent::Command(_, actor_key, command_type) => match command_type {
                            ExampleEvent::KeyCommand(key_command) => {
                                if let Some(typed_actor) = server.get_actor(actor_key) {
                                    match typed_actor {
                                        ExampleActor::PointActor(actor) => {
                                            shared_behavior::process_command(&key_command, actor);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        ServerEvent::Tick => {
                            server.send_all_updates().await;
                            //info!("tick");
                        }
                        _ => {}
                    }
                }
                Err(error) => {
                    info!("Naia Server Error: {}", error);
                }
            }
        }
    })
}
