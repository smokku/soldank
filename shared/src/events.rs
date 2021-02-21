use naia_derive::EventType;

#[derive(EventType, Clone)]
pub enum NetworkEvent {
    KeyCommand(key::KeyCommand),
    AuthEvent(auth::AuthEvent),
}

pub mod auth {
    use super::NetworkEvent;
    use naia_derive::Event;
    use naia_shared::{Event, Property};

    #[derive(Event, Clone)]
    #[type_name = "NetworkEvent"]
    pub struct AuthEvent {
        pub nick: Property<String>,
        pub key: Property<String>,
    }

    impl AuthEvent {
        fn is_guaranteed() -> bool {
            true
        }

        pub fn new(nick: &str, key: &str) -> AuthEvent {
            return AuthEvent::new_complete(nick.to_string(), key.to_string());
        }
    }
}

pub mod key {
    use super::NetworkEvent;
    use naia_derive::Event;
    use naia_shared::{Event, Property};

    #[derive(Event, Clone)]
    #[type_name = "NetworkEvent"]
    pub struct KeyCommand {
        pub w: Property<bool>,
        pub s: Property<bool>,
        pub a: Property<bool>,
        pub d: Property<bool>,
    }

    impl KeyCommand {
        fn is_guaranteed() -> bool {
            false
        }

        pub fn new(w: bool, s: bool, a: bool, d: bool) -> KeyCommand {
            return KeyCommand::new_complete(w, s, a, d);
        }
    }
}
