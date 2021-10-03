use crate::{
    engine::input::{InputEngine, InputState},
    mq,
};
use cvar::IVisit;
use std::collections::HashMap;

pub struct ScriptEngine {
    vars: HashMap<String, String>,
    commands: HashMap<&'static str, (usize, CommandFunction)>,
}

struct Env<'a> {
    input: &'a mut InputEngine,
    config: &'a mut dyn IVisit,
}

type CommandFunction = fn(&[&str], &mut Env) -> Result<Option<String>, String>;

impl ScriptEngine {
    pub fn new() -> ScriptEngine {
        let mut commands = HashMap::new();

        commands.insert("get", (1, cvars_get as CommandFunction));
        commands.insert("set", (2, cvars_set as CommandFunction));
        commands.insert("toggle", (1, cvars_toggle as CommandFunction));
        commands.insert("log", (1, log_info as CommandFunction));
        commands.insert("warn", (1, log_warn as CommandFunction));
        commands.insert("error", (1, log_error as CommandFunction));
        commands.insert("bind", (1, bind_key as CommandFunction));
        commands.insert("unbind", (1, unbind_key as CommandFunction));

        ScriptEngine {
            vars: HashMap::new(),
            commands,
        }
    }

    pub fn evaluate<S: Into<String>>(
        &mut self,
        script: S,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
    ) -> Result<(), String> {
        for (i, line) in script.into().lines().enumerate() {
            let i = i + 1; // files start counting lines with line 1
            let mut words: Vec<String> = line
                .split_ascii_whitespace()
                .take_while(|word| !word.starts_with('#'))
                .map(String::from)
                .collect();

            if words.is_empty() {
                continue;
            }

            let mut idx = 0;
            let mut assignment = false;
            let result;

            if words.len() > 2 && words[1] == "=" {
                // variable assignment
                assignment = true;
                idx = 2;
            }

            if words.len() <= idx {
                return Err(format!("Command missing. Line: {}", i));
            }

            for i in 0..words.len() {
                if words[i].starts_with('$') {
                    if let Some(value) = self.vars.get(&words[i][1..]) {
                        words[i] = (*value).clone();
                    } else {
                        return Err(format!("Variable unset: {}. Line: {}", words[i], i));
                    }
                }
            }
            let words: Vec<&str> = words.iter().map(String::as_str).collect();

            let command = words[idx];
            idx += 1;

            if let Some((min_args, cmd)) = self.commands.get(command) {
                if words.len() - idx < *min_args {
                    return Err(format!("Not enough arguments. Line: {}", i));
                } else {
                    match cmd(&words[idx..], &mut Env { config, input }) {
                        Ok(res) => result = res,
                        Err(err) => {
                            return Err(format!("{} Line: {}", err, i));
                        }
                    }
                }
            } else {
                return Err(format!("Unknown command: {}. Line: {}", command, i));
            }

            if assignment {
                if let Some(result) = result {
                    self.vars.insert(words[0].to_string(), result);
                } else {
                    self.vars.remove(words[0]);
                }
            } else if result.is_some() {
                return Err(format!("Unused result. Line: {}", i));
            }
        }

        Ok(())
    }

    pub fn evaluate_file<F: std::io::Read>(
        &mut self,
        mut file: F,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
    ) -> Result<(), String> {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Error reading File");
        self.evaluate(String::from_utf8_lossy(&buffer).as_ref(), input, config)
    }
}

fn cvars_get(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    Ok(cvar::console::get(env.config, args[0]))
}

fn cvars_set(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    match cvar::console::set(env.config, args[0], args[1]).unwrap() {
        true => Ok(None),
        false => Err("No such cvar.".to_string()),
    }
}

fn cvars_toggle(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let mut output = Err("No such cvar.".to_string());

    cvar::console::find(env.config, args[0], |node| {
        if let cvar::Node::Prop(prop) = node.as_node() {
            match prop.get().as_str() {
                "true" => {
                    if prop.set("false").is_ok() {
                        output = Ok(None);
                    }
                }
                "false" => {
                    if prop.set("true").is_ok() {
                        output = Ok(None);
                    }
                }
                _ => {
                    output = Err("Value of cvar is not boolean".to_string());
                }
            }
        }
    });

    output
}

fn log_info(args: &[&str], _env: &mut Env) -> Result<Option<String>, String> {
    log::info!("{}", args.join(" "));
    Ok(None)
}
fn log_error(args: &[&str], _env: &mut Env) -> Result<Option<String>, String> {
    log::error!("{}", args.join(" "));
    Ok(None)
}
fn log_warn(args: &[&str], _env: &mut Env) -> Result<Option<String>, String> {
    log::warn!("{}", args.join(" "));
    Ok(None)
}

fn bind_key(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    match keycode_from_str(args[0]) {
        Ok(kc) => {
            if args[1].starts_with('+') {
                if let Ok(state) = input_from_str(&args[1][1..]) {
                    env.input.bind_key(kc, state);
                    return Ok(None);
                }
            }
            Err("Unknown input state.".to_string())
        }
        Err(_) => Err("Unknown keycode.".to_string()),
    }
}

fn unbind_key(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    match keycode_from_str(args[0]) {
        Ok(kc) => {
            env.input.unbind_key(kc);
            Ok(None)
        }
        Err(_) => Err("Unknown keycode.".to_string()),
    }
}

fn keycode_from_str(input: &str) -> Result<mq::KeyCode, ()> {
    match input.to_lowercase().as_str() {
        "space" => Ok(mq::KeyCode::Space),
        "apostrophe" => Ok(mq::KeyCode::Apostrophe),
        "comma" => Ok(mq::KeyCode::Comma),
        "minus" => Ok(mq::KeyCode::Minus),
        "period" => Ok(mq::KeyCode::Period),
        "slash" => Ok(mq::KeyCode::Slash),
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
        "semicolon" => Ok(mq::KeyCode::Semicolon),
        "equal" => Ok(mq::KeyCode::Equal),
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
        "backslash" => Ok(mq::KeyCode::Backslash),
        "rightbracket" => Ok(mq::KeyCode::RightBracket),
        "graveaccent" => Ok(mq::KeyCode::GraveAccent),
        "world1" => Ok(mq::KeyCode::World1),
        "world2" => Ok(mq::KeyCode::World2),
        "escape" => Ok(mq::KeyCode::Escape),
        "enter" => Ok(mq::KeyCode::Enter),
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

fn input_from_str(input: &str) -> Result<InputState, ()> {
    match input.to_lowercase().as_str() {
        "moveleft" => Ok(InputState::MoveLeft),
        "moveright" => Ok(InputState::MoveRight),
        "jump" => Ok(InputState::Jump),
        "crouch" => Ok(InputState::Crouch),
        "prone" => Ok(InputState::Prone),
        _ => Err(()),
    }
}
