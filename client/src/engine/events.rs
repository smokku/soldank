#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    ConfigChanged,
    Command(String),
}
