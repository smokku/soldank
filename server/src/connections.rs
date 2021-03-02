#[derive(Debug)]
pub struct Connection {
    pub last_message_received: f64,
    pub authorized: bool,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            last_message_received: instant::now(),
            authorized: false,
        }
    }
}
