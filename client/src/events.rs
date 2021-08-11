#[derive(Debug, PartialEq)]
pub enum AppEvent {
    CvarsChanged,
}

pub type AppEventsQueue = Vec<AppEvent>;
