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
    pub(crate) binds: HashMap<KeyBind, Vec<BindEntry>>,
}

#[derive(Debug)]
pub enum BindEntry {
    State(KeyMods, BitFlags<InputState>),
    Script(KeyMods, String),
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

    pub fn bind_key<F: Into<BitFlags<InputState>>>(
        &mut self,
        key: KeyBind,
        mods: KeyMods,
        inputs: F,
    ) {
        let bind = self.binds.entry(key).or_default();
        bind.push(BindEntry::State(mods, inputs.into()));
    }

    pub fn bind_script<S: Into<String>>(&mut self, key: KeyBind, mods: KeyMods, script: S) {
        let bind = self.binds.entry(key).or_default();
        bind.push(BindEntry::Script(mods, script.into()));
    }

    pub fn unbind_key(&mut self, key: KeyBind) {
        self.binds.remove(&key);
    }

    pub fn unbind_all(&mut self) {
        self.binds.clear();
    }
}

#[inline]
fn check_mods(mods: KeyMods, keymods: mq::KeyMods) -> bool {
    (!mods.shift || keymods.shift) && (!mods.alt || keymods.alt) && (!mods.ctrl || keymods.ctrl)
}

impl<G: Game> Runner<G> {
    pub fn handle_bind(&mut self, key: &KeyBind, keymods: mq::KeyMods, down: bool) {
        if let Some(binds) = self.input.binds.get(key) {
            for bind in binds {
                match &bind {
                    BindEntry::State(mods, state) => {
                        if down {
                            if check_mods(*mods, keymods) {
                                self.input.state.insert(*state);
                            }
                        } else {
                            self.input.state.remove(*state);
                        }
                    }
                    BindEntry::Script(mods, script) => {
                        if down && check_mods(*mods, keymods) {
                            if let Err(err) =
                                self.event_sender.try_send(Event::Command(script.clone()))
                            {
                                log::error!("Cannot send Command Event: {}", err);
                            }
                        }
                    }
                }
            }
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyBind {
    Key(mq::KeyCode),
    Mouse(mq::MouseButton),
    Button(),
}

#[derive(Default, Debug, Copy, Clone)]
pub struct KeyMods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl std::str::FromStr for KeyBind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        keycode_from_str(s).map(KeyBind::Key).map_err(|err| err)
    }
}

fn keycode_from_str(input: &str) -> Result<mq::KeyCode, ()> {
    match input.to_ascii_lowercase().as_str() {
        "space" => Ok(mq::KeyCode::Space),
        " " => Ok(mq::KeyCode::Space),
        "apostrophe" => Ok(mq::KeyCode::Apostrophe),
        "'" => Ok(mq::KeyCode::Apostrophe),
        "comma" => Ok(mq::KeyCode::Comma),
        "," => Ok(mq::KeyCode::Comma),
        "minus" => Ok(mq::KeyCode::Minus),
        "-" => Ok(mq::KeyCode::Minus),
        "period" => Ok(mq::KeyCode::Period),
        "." => Ok(mq::KeyCode::Period),
        "slash" => Ok(mq::KeyCode::Slash),
        "/" => Ok(mq::KeyCode::Slash),
        "key0" => Ok(mq::KeyCode::Key0),
        "key1" => Ok(mq::KeyCode::Key1),
        "key2" => Ok(mq::KeyCode::Key2),
        "key3" => Ok(mq::KeyCode::Key3),
        "key4" => Ok(mq::KeyCode::Key4),
        "key5" => Ok(mq::KeyCode::Key5),
        "key6" => Ok(mq::KeyCode::Key6),
        "key7" => Ok(mq::KeyCode::Key7),
        "key8" => Ok(mq::KeyCode::Key8),
        "key9" => Ok(mq::KeyCode::Key9),
        "0" => Ok(mq::KeyCode::Key0),
        "1" => Ok(mq::KeyCode::Key1),
        "2" => Ok(mq::KeyCode::Key2),
        "3" => Ok(mq::KeyCode::Key3),
        "4" => Ok(mq::KeyCode::Key4),
        "5" => Ok(mq::KeyCode::Key5),
        "6" => Ok(mq::KeyCode::Key6),
        "7" => Ok(mq::KeyCode::Key7),
        "8" => Ok(mq::KeyCode::Key8),
        "9" => Ok(mq::KeyCode::Key9),
        "semicolon" => Ok(mq::KeyCode::Semicolon),
        ";" => Ok(mq::KeyCode::Semicolon),
        "equal" => Ok(mq::KeyCode::Equal),
        "=" => Ok(mq::KeyCode::Equal),
        "a" => Ok(mq::KeyCode::A),
        "b" => Ok(mq::KeyCode::B),
        "c" => Ok(mq::KeyCode::C),
        "d" => Ok(mq::KeyCode::D),
        "e" => Ok(mq::KeyCode::E),
        "f" => Ok(mq::KeyCode::F),
        "g" => Ok(mq::KeyCode::G),
        "h" => Ok(mq::KeyCode::H),
        "i" => Ok(mq::KeyCode::I),
        "j" => Ok(mq::KeyCode::J),
        "k" => Ok(mq::KeyCode::K),
        "l" => Ok(mq::KeyCode::L),
        "m" => Ok(mq::KeyCode::M),
        "n" => Ok(mq::KeyCode::N),
        "o" => Ok(mq::KeyCode::O),
        "p" => Ok(mq::KeyCode::P),
        "q" => Ok(mq::KeyCode::Q),
        "r" => Ok(mq::KeyCode::R),
        "s" => Ok(mq::KeyCode::S),
        "t" => Ok(mq::KeyCode::T),
        "u" => Ok(mq::KeyCode::U),
        "v" => Ok(mq::KeyCode::V),
        "w" => Ok(mq::KeyCode::W),
        "x" => Ok(mq::KeyCode::X),
        "y" => Ok(mq::KeyCode::Y),
        "z" => Ok(mq::KeyCode::Z),
        "leftbracket" => Ok(mq::KeyCode::LeftBracket),
        "[" => Ok(mq::KeyCode::LeftBracket),
        "backslash" => Ok(mq::KeyCode::Backslash),
        "\\" => Ok(mq::KeyCode::Backslash),
        "rightbracket" => Ok(mq::KeyCode::RightBracket),
        "]" => Ok(mq::KeyCode::RightBracket),
        "graveaccent" => Ok(mq::KeyCode::GraveAccent),
        "`" => Ok(mq::KeyCode::GraveAccent),
        "world1" => Ok(mq::KeyCode::World1),
        "world2" => Ok(mq::KeyCode::World2),
        "escape" => Ok(mq::KeyCode::Escape),
        "esc" => Ok(mq::KeyCode::Escape),
        "enter" => Ok(mq::KeyCode::Enter),
        "return" => Ok(mq::KeyCode::Enter),
        "tab" => Ok(mq::KeyCode::Tab),
        "backspace" => Ok(mq::KeyCode::Backspace),
        "insert" => Ok(mq::KeyCode::Insert),
        "delete" => Ok(mq::KeyCode::Delete),
        "right" => Ok(mq::KeyCode::Right),
        "left" => Ok(mq::KeyCode::Left),
        "down" => Ok(mq::KeyCode::Down),
        "up" => Ok(mq::KeyCode::Up),
        "pageup" => Ok(mq::KeyCode::PageUp),
        "pagedown" => Ok(mq::KeyCode::PageDown),
        "home" => Ok(mq::KeyCode::Home),
        "end" => Ok(mq::KeyCode::End),
        "capslock" => Ok(mq::KeyCode::CapsLock),
        "scrolllock" => Ok(mq::KeyCode::ScrollLock),
        "numlock" => Ok(mq::KeyCode::NumLock),
        "printscreen" => Ok(mq::KeyCode::PrintScreen),
        "pause" => Ok(mq::KeyCode::Pause),
        "f1" => Ok(mq::KeyCode::F1),
        "f2" => Ok(mq::KeyCode::F2),
        "f3" => Ok(mq::KeyCode::F3),
        "f4" => Ok(mq::KeyCode::F4),
        "f5" => Ok(mq::KeyCode::F5),
        "f6" => Ok(mq::KeyCode::F6),
        "f7" => Ok(mq::KeyCode::F7),
        "f8" => Ok(mq::KeyCode::F8),
        "f9" => Ok(mq::KeyCode::F9),
        "f10" => Ok(mq::KeyCode::F10),
        "f11" => Ok(mq::KeyCode::F11),
        "f12" => Ok(mq::KeyCode::F12),
        "f13" => Ok(mq::KeyCode::F13),
        "f14" => Ok(mq::KeyCode::F14),
        "f15" => Ok(mq::KeyCode::F15),
        "f16" => Ok(mq::KeyCode::F16),
        "f17" => Ok(mq::KeyCode::F17),
        "f18" => Ok(mq::KeyCode::F18),
        "f19" => Ok(mq::KeyCode::F19),
        "f20" => Ok(mq::KeyCode::F20),
        "f21" => Ok(mq::KeyCode::F21),
        "f22" => Ok(mq::KeyCode::F22),
        "f23" => Ok(mq::KeyCode::F23),
        "f24" => Ok(mq::KeyCode::F24),
        "f25" => Ok(mq::KeyCode::F25),
        "kp0" => Ok(mq::KeyCode::Kp0),
        "kp1" => Ok(mq::KeyCode::Kp1),
        "kp2" => Ok(mq::KeyCode::Kp2),
        "kp3" => Ok(mq::KeyCode::Kp3),
        "kp4" => Ok(mq::KeyCode::Kp4),
        "kp5" => Ok(mq::KeyCode::Kp5),
        "kp6" => Ok(mq::KeyCode::Kp6),
        "kp7" => Ok(mq::KeyCode::Kp7),
        "kp8" => Ok(mq::KeyCode::Kp8),
        "kp9" => Ok(mq::KeyCode::Kp9),
        "kpdecimal" => Ok(mq::KeyCode::KpDecimal),
        "kpdivide" => Ok(mq::KeyCode::KpDivide),
        "kpmultiply" => Ok(mq::KeyCode::KpMultiply),
        "kpsubtract" => Ok(mq::KeyCode::KpSubtract),
        "kpadd" => Ok(mq::KeyCode::KpAdd),
        "kpenter" => Ok(mq::KeyCode::KpEnter),
        "kpequal" => Ok(mq::KeyCode::KpEqual),
        "leftshift" => Ok(mq::KeyCode::LeftShift),
        "leftcontrol" => Ok(mq::KeyCode::LeftControl),
        "leftalt" => Ok(mq::KeyCode::LeftAlt),
        "leftsuper" => Ok(mq::KeyCode::LeftSuper),
        "rightshift" => Ok(mq::KeyCode::RightShift),
        "rightcontrol" => Ok(mq::KeyCode::RightControl),
        "rightalt" => Ok(mq::KeyCode::RightAlt),
        "rightsuper" => Ok(mq::KeyCode::RightSuper),
        "menu" => Ok(mq::KeyCode::Menu),
        "unknown" => Ok(mq::KeyCode::Unknown),

        _ => Err(()),
    }
}
