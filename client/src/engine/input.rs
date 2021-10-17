use super::*;
use enumflags2::{bitflags, BitFlags};
use std::{
    collections::{vec_deque::Drain, HashMap, VecDeque},
    str::FromStr,
};

#[derive(Debug)]
pub enum InputEvent {
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
    pub(crate) queue: VecDeque<InputEvent>,
    pub state: BitFlags<InputState>,
    pub(crate) binds: HashMap<mq::KeyCode, BitFlags<InputState>>,
}

impl InputEngine {
    pub fn new() -> Self {
        InputEngine {
            mouse_x: 0.0,
            mouse_y: 0.0,
            queue: VecDeque::new(),
            state: Default::default(),
            binds: HashMap::new(),
        }
    }

    pub(crate) fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn add_event(&mut self, event: InputEvent) {
        self.queue.push_back(event);
    }

    pub fn drain_events(&mut self) -> Drain<'_, InputEvent> {
        self.queue.drain(..)
    }

    pub fn bind_key<F: Into<BitFlags<InputState>>>(&mut self, key: mq::KeyCode, inputs: F) {
        let bind = self.binds.entry(key).or_default();
        bind.insert(inputs);
    }

    pub fn unbind_key(&mut self, key: mq::KeyCode) {
        self.binds.remove(&key);
    }

    pub fn unbind_all(&mut self) {
        self.binds.clear();
    }
}

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputState {
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Prone,
    Fire,
    Jet,
    ChangeWeapon,
    Reload,
    DropWeapon,
    ThrowGrenade,
    Chat,
    TeamChat,
    Radio,
    Weapons,
    FragsList,
    StatsMenu,
    MiniMap,
    Cmd,
    GameStats,
}

impl FromStr for InputState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(InputState::MoveLeft),
            "moveleft" => Ok(InputState::MoveLeft),
            "right" => Ok(InputState::MoveRight),
            "moveright" => Ok(InputState::MoveRight),
            "jump" => Ok(InputState::Jump),
            "crouch" => Ok(InputState::Crouch),
            "prone" => Ok(InputState::Prone),
            "fire" => Ok(InputState::Fire),
            "jet" => Ok(InputState::Jet),
            "changeweapon" => Ok(InputState::ChangeWeapon),
            "reload" => Ok(InputState::Reload),
            "dropweapon" => Ok(InputState::DropWeapon),
            "throwgrenade" => Ok(InputState::ThrowGrenade),
            "chat" => Ok(InputState::Chat),
            "teamchat" => Ok(InputState::TeamChat),
            "radio" => Ok(InputState::Radio),
            "weapons" => Ok(InputState::Weapons),
            "fragslist" => Ok(InputState::FragsList),
            "statsmenu" => Ok(InputState::StatsMenu),
            "minimap" => Ok(InputState::MiniMap),
            "cmd" => Ok(InputState::Cmd),
            "gamestats" => Ok(InputState::GameStats),
            _ => Err(()),
        }
    }
}
