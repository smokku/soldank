#[derive(Debug)]
pub struct Connection {
    pub last_message_received: f64,
    pub authorized: bool,
    pub nick: String,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            last_message_received: instant::now(),
            authorized: false,
            nick: Default::default(),
        }
    }
}
