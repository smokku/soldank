use super::*;
use std::collections::{vec_deque::Drain, VecDeque};

#[derive(Debug)]
pub enum Input {
    Key {
        down: bool,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        repeat: bool,
    },
    Mouse {
        down: bool,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    },
    Wheel {
        dx: f32,
        dy: f32,
    },
}

#[derive(Debug, Default)]
pub struct InputEngine {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub(crate) queue: VecDeque<Input>,
}

impl InputEngine {
    pub fn new() -> Self {
        InputEngine {
            mouse_x: 0.0,
            mouse_y: 0.0,
            queue: VecDeque::new(),
        }
    }

    pub(crate) fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn add_event(&mut self, event: Input) {
        self.queue.push_back(event);
    }

    pub fn drain_events(&mut self) -> Drain<'_, Input> {
        self.queue.drain(..)
    }
}
