use crate::engine::{
    input::{InputEngine, InputState},
    utils::*,
};
use cvar::IVisit;
use gvfs::filesystem::Filesystem;
use rhai::{
    packages::{Package, StandardPackage},
    plugin::*,
    Dynamic, Engine, Scope,
};
use std::{cell::RefCell, collections::HashMap, io::Read, rc::Rc, str::FromStr};

pub struct ScriptEngine {
    vars: HashMap<String, String>,
    commands: HashMap<&'static str, (usize, CommandFunction)>,
    engine: Engine,
}

struct Env<'a> {
    input: &'a mut InputEngine,
    config: &'a mut dyn IVisit,
    fs: &'a mut Filesystem,
    engine: &'a mut Engine,
}

type CommandFunction = fn(&[&str], &mut Env) -> Result<Option<String>, String>;

pub struct WorldEnv {}

pub type SharedWorld = Rc<RefCell<WorldEnv>>;

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
        commands.insert("echo", (0, echo_args as CommandFunction));
        commands.insert("eval", (1, eval_rhai as CommandFunction));
        commands.insert("run", (1, run_rhai as CommandFunction));

        let mut engine = Engine::new_raw();

        engine.on_print(|text| log::info!("{}", text));

        engine.on_debug(|text, source, pos| {
            if let Some(source) = source {
                log::debug!("{}:{:?} | {}", source, pos, text);
            } else if pos.is_none() {
                log::debug!("{}", text);
            } else {
                log::debug!("{:?} | {}", pos, text);
            }
        });

        let package = StandardPackage::new().as_shared_module();
        engine.register_global_module(package);

        engine.register_type_with_name::<SharedWorld>("World");
        engine.register_global_module(exported_module!(world_api).into());

        ScriptEngine {
            vars: HashMap::new(),
            commands,
            engine,
        }
    }

    pub fn evaluate<S: Into<String>>(
        &mut self,
        script: S,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
        fs: &mut Filesystem,
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

            for word in words.iter_mut().skip(idx) {
                if word.starts_with('$') {
                    if let Some(value) = self.vars.get(&word[1..]) {
                        *word = value.clone();
                    } else {
                        return Err(format!("Variable unset: {}. Line: {}", word, i));
                    }
                }
            }
            let words: Vec<&str> = words.iter().map(String::as_str).collect();

            // We already have variable replacements here, so it is possible
            // to have command name resolved from variable
            let command = words[idx];
            idx += 1;

            if let Some((min_args, cmd)) = self.commands.get(command) {
                if words.len() - idx < *min_args {
                    return Err(format!("Not enough arguments. Line: {}", i));
                } else {
                    match cmd(
                        &words[idx..],
                        &mut Env {
                            config,
                            fs,
                            input,
                            engine: &mut self.engine,
                        },
                    ) {
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

    pub fn evaluate_file<S: AsRef<str>>(
        &mut self,
        file: S,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
        fs: &mut Filesystem,
    ) -> Result<(), String> {
        let mut file = fs.open(file.as_ref()).map_err(|err| err.to_string())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|err| err.to_string())?;
        self.evaluate(String::from_utf8_lossy(&buffer).as_ref(), input, config, fs)
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
                if let Ok(state) = InputState::from_str(&args[1][1..]) {
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

fn echo_args(args: &[&str], _env: &mut Env) -> Result<Option<String>, String> {
    if args.is_empty() {
        Ok(None)
    } else {
        Ok(Some(args.join(" ")))
    }
}

fn eval_rhai(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let res = env
        .engine
        .eval_expression::<Dynamic>(args.join(" ").as_str())
        .map_err(|err| err.to_string())?;

    Ok(Some(res.to_string()))
}

fn run_rhai(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let script = args[0];
    if !script.ends_with(".rhai") {
        return Err("Script must end with .rhai extension.".to_string());
    }

    let mut file = env.fs.open(script).map_err(|err| err.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;

    let mut ast = env
        .engine
        .compile(String::from_utf8_lossy(&buffer).as_ref())
        .map_err(|err| {
            log::error!("Failed to compile {}: {}", script, err);
            format!("Failed to compile {}.", script)
        })?;
    ast.set_source(script);
    let world: SharedWorld = Rc::new(RefCell::new(WorldEnv::new()));
    let mut scope = Scope::new();
    scope.push_constant("World", world.clone());
    env.engine
        .consume_ast_with_scope(&mut scope, &ast)
        .map_err(|err| {
            log::error!("Error running {}: {}", script, err);
            format!("Error running {}.", script)
        })?;

    Ok(None)
}

impl WorldEnv {
    fn new() -> Self {
        WorldEnv {}
    }

    fn len(&self) -> i32 {
        42
    }
}

#[export_module]
mod world_api {
    #[rhai_fn(get = "len", pure)]
    pub fn get_len(world: &mut SharedWorld) -> i32 {
        world.borrow().len()
    }
}
