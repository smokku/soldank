use super::*;
use crate::engine::input::{InputEngine, InputState};
use cvar::IVisit;
use gvfs::filesystem::Filesystem;
use rhai::{
    packages::{Package, StandardPackage},
    plugin::*,
    Dynamic, Engine, Scope, AST,
};
use std::{collections::HashMap, fmt, io::Read, str::FromStr};

pub struct ScriptEngine {
    vars: HashMap<String, String>,
    commands: HashMap<&'static str, (usize, CommandFunction)>,

    engine: Engine,
    ast: HashMap<String, AST>,

    event_send: BroadcastSender<Event>,
    event_recv: BroadcastReceiver<Event>,
}

pub struct ScriptError {
    source: Option<String>,
    line: Option<usize>,
    message: String,
}

impl ScriptError {
    fn from_line<S: Into<String>>(message: S, line: usize) -> Self {
        ScriptError {
            source: None,
            line: Some(line),
            message: message.into(),
        }
    }

    fn from_source<S: Into<String>>(message: S, source: S) -> Self {
        ScriptError {
            source: Some(source.into()),
            line: None,
            message: message.into(),
        }
    }

    fn set_source<S: Into<String>>(&mut self, source: S) {
        self.source.replace(source.into());
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(source) = &self.source {
            write!(
                f,
                "{}{} | {}",
                source,
                self.line
                    .map_or_else(String::new, |line| { format!(":{}", line) }),
                self.message
            )
        } else if self.line.is_none() {
            write!(f, "{}", self.message)
        } else {
            write!(f, ":{} | {}", self.line.unwrap(), self.message)
        }
    }
}

struct Env<'a> {
    input: &'a mut InputEngine,
    config: &'a mut dyn IVisit,
    fs: &'a mut Filesystem,
    vars: &'a mut HashMap<String, String>,
    engine: &'a mut Engine,
    ast: &'a mut HashMap<String, AST>,
    world: &'a mut hecs::World,
    event_sender: &'a BroadcastSender<Event>,
}

type CommandFunction = fn(&[&str], &mut Env) -> Result<Option<String>, String>;

#[derive(Clone)]
pub struct WorldEnv {
    inner: *mut hecs::World,
}

impl ScriptEngine {
    pub fn new(
        event_send: BroadcastSender<Event>,
        event_recv: BroadcastReceiver<Event>,
    ) -> ScriptEngine {
        let mut commands = HashMap::new();

        commands.insert("exec", (1, fake_command as CommandFunction));
        commands.insert("get", (1, cvars_get as CommandFunction));
        commands.insert("set", (2, cvars_set as CommandFunction));
        commands.insert("toggle", (1, cvars_toggle as CommandFunction));
        commands.insert("log", (1, log_info as CommandFunction));
        commands.insert("warn", (1, log_warn as CommandFunction));
        commands.insert("error", (1, log_error as CommandFunction));
        commands.insert("bind", (1, bind_key as CommandFunction));
        commands.insert("unbind", (1, unbind_key as CommandFunction));
        commands.insert("unbindall", (0, unbind_all as CommandFunction));
        commands.insert("echo", (0, echo_args as CommandFunction));
        commands.insert("exit", (0, exit_game as CommandFunction));
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

        engine.register_type_with_name::<WorldEnv>("World");
        engine.register_global_module(exported_module!(world_api).into());

        ScriptEngine {
            vars: HashMap::new(),
            commands,

            engine,
            ast: HashMap::new(),

            event_send,
            event_recv,
        }
    }

    pub fn evaluate<S: Into<String>>(
        &mut self,
        script: S,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
        fs: &mut Filesystem,
        world: &mut hecs::World,
    ) -> Result<(), ScriptError> {
        // Reset internal state
        self.vars.insert("IFS".to_string(), " ".to_string());
        self.vars.insert("NL".to_string(), "\n".to_string());
        self.vars.insert("TAB".to_string(), "\t".to_string());
        self.vars.insert("SPACE".to_string(), " ".to_string());

        for (i, line) in script.into().lines().enumerate() {
            let i = i + 1; // files start counting lines with line 1
            let mut words: Vec<String> = line
                .split_ascii_whitespace()
                .take_while(|word| !word.starts_with('#') && !word.starts_with("//"))
                .map(|s| {
                    let mut s = String::from(s);
                    if s.starts_with('"') && s.ends_with('"') {
                        s.remove(0);
                        s.pop();
                    }
                    s
                })
                .collect();

            if words.is_empty() {
                continue;
            }

            let mut idx = 0;
            let mut assignment = false;
            let mut result = None;

            if words.len() > 2 && words[1] == "=" {
                // variable assignment
                assignment = true;
                idx = 2;
            }

            if words.len() <= idx {
                return Err(ScriptError::from_line("Command missing.", i));
            }

            for word in words.iter_mut().skip(idx) {
                if word.starts_with('$') {
                    if let Some(value) = self.vars.get(&word[1..]) {
                        *word = value.clone();
                    } else {
                        return Err(ScriptError::from_line(
                            format!("Variable unset: {}.", word),
                            i,
                        ));
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
                    return Err(ScriptError::from_line("Not enough arguments.", i));
                } else {
                    let args = &words[idx..];
                    if command == "exec" {
                        if args.is_empty() || !args[0].ends_with(".cfg") {
                            return Err(ScriptError::from_line(
                                "Script must end with .cfg extension.".to_string(),
                                i,
                            ));
                        }
                        if let Err(err) = self.evaluate_file(args[0], input, config, fs, world) {
                            return Err(err);
                        }
                    } else {
                        match cmd(
                            args,
                            &mut Env {
                                config,
                                fs,
                                input,
                                vars: &mut self.vars,
                                engine: &mut self.engine,
                                ast: &mut self.ast,
                                world,
                                event_sender: &self.event_send,
                            },
                        ) {
                            Ok(res) => result = res,
                            Err(err) => {
                                return Err(ScriptError::from_line(err, i));
                            }
                        }
                    }
                }
            } else {
                return Err(ScriptError::from_line(
                    format!("Unknown command: {:?}.", command),
                    i,
                ));
            }

            if assignment {
                if let Some(result) = result {
                    self.vars.insert(words[0].to_string(), result);
                } else {
                    self.vars.remove(words[0]);
                }
            } else if result.is_some() {
                return Err(ScriptError::from_line("Unused result.", i));
            }
        }

        Ok(())
    }

    pub fn evaluate_file<S: AsRef<str>>(
        &mut self,
        file_name: S,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
        fs: &mut Filesystem,
        world: &mut hecs::World,
    ) -> Result<(), ScriptError> {
        let file_name = file_name.as_ref();
        let mut file = fs
            .open(file_name)
            .map_err(|err| ScriptError::from_source(err.to_string(), file_name.to_string()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|err| ScriptError::from_source(err.to_string(), file_name.to_string()))?;
        self.evaluate(
            String::from_utf8_lossy(&buffer).as_ref(),
            input,
            config,
            fs,
            world,
        )
        .map_err(|mut err| {
            if err.source.is_none() {
                err.set_source(file_name)
            };
            err
        })
    }

    pub(crate) fn consume_events(
        &mut self,
        input: &mut InputEngine,
        config: &mut dyn IVisit,
        fs: &mut Filesystem,
        world: &mut hecs::World,
    ) {
        let mut commands = Vec::new();
        for event in &self.event_recv {
            log::trace!("ScriptEngine Event consumer got {:?}", event);
            if let Event::Command(script) = event {
                commands.push(script);
            }
        }
        for command in commands {
            if let Err(err) = self.evaluate(command, input, config, fs, world) {
                log::error!("Error evaluating Command Event: {}", err);
            }
        }
    }

    pub(crate) fn drain_events(&mut self) {
        for event in &self.event_recv {
            if let Event::Command(script) = event {
                log::error!("Unhandled Command Event: {}\nForgot to call ScriptEngine::consume_events() in Game::update()?", script);
            }
        }
    }
}

fn fake_command(_args: &[&str], _env: &mut Env) -> Result<Option<String>, String> {
    Err("Called fake command.".to_string())
}

fn cvars_get(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let var = args[0];
    if var.starts_with("--") {
        match var {
            "--dump" => {
                let mut dump = Vec::new();
                cvar::console::walk(env.config, |path, node| {
                    if let cvar::Node::Prop(prop) = node.as_node() {
                        dump.push(format!("{} = `{}`", path, prop.get()));
                    }
                });
                Ok(Some(dump.join("\n")))
            }
            _ => Err(format!("Unknown option: {:?}.", var)),
        }
    } else {
        Ok(cvar::console::get(env.config, args[0]))
    }
}

fn cvars_set(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    match cvar::console::set(env.config, args[0], args[1]).unwrap() {
        true => {
            if let Err(err) = env.event_sender.try_send(Event::ConfigChanged) {
                log::error!("Cannot send ConfigChanged Event: {}", err);
            }

            Ok(None)
        }
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

    if output.is_ok() {
        if let Err(err) = env.event_sender.try_send(Event::ConfigChanged) {
            log::error!("Cannot send ConfigChanged Event: {}", err);
        }
    }

    output
}

fn join_args(args: &[&str], env: &mut Env) -> String {
    args.join(env.vars.get("IFS").unwrap_or(&"".to_string()))
}

fn log_info(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    log::info!("{}", join_args(args, env));
    Ok(None)
}
fn log_error(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    log::error!("{}", join_args(args, env));
    Ok(None)
}
fn log_warn(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    log::warn!("{}", join_args(args, env));
    Ok(None)
}

fn bind_key(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let key = args[0].to_ascii_lowercase();
    let mut mods = KeyMods::default();

    let kb = if let Some(button) = key.strip_prefix("mouse") {
        KeyBind::Mouse(match button {
            "1" => mq::MouseButton::Left,
            "2" => mq::MouseButton::Middle,
            "3" => mq::MouseButton::Right,
            _ => return Err("Unknown mouse button".to_string()),
        })
    } else {
        let mut rest = "";
        for key in key.split('+') {
            match key {
                "shift" => mods.shift = true,
                "ctrl" => mods.ctrl = true,
                "alt" => mods.alt = true,
                k => {
                    rest = k;
                    break;
                }
            }
        }
        KeyBind::from_str(rest).map_err(|_err| "Unknown keycode.".to_string())?
    };

    if args[1].starts_with('+') {
        if let Ok(state) = InputState::from_str(&args[1][1..]) {
            env.input.bind_key(kb, mods, state);
            return Ok(None);
        }
        Err("Unknown input state.".to_string())
    } else {
        let mut script = args[1..].join(" ");
        if script.starts_with('"') && script.ends_with('"') {
            script.remove(0);
            script.pop();
        }
        env.input.bind_script(kb, mods, script);
        Ok(None)
    }
}

fn unbind_key(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    match KeyBind::from_str(args[0]) {
        Ok(kc) => {
            env.input.unbind_key(kc);
            Ok(None)
        }
        Err(_) => Err("Unknown keycode.".to_string()),
    }
}

fn unbind_all(_args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    env.input.unbind_all();
    Ok(None)
}

fn echo_args(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    if args.is_empty() {
        Ok(None)
    } else {
        Ok(Some(join_args(args, env)))
    }
}

fn exit_game(_args: &[&str], _env: &mut Env) -> Result<Option<String>, String> {
    if cfg!(debug_assertions) {
        log::info!("Script exit");
        std::process::abort();
    } else {
        log::warn!("Attempted script exit!");
    }
    Ok(None)
}

fn eval_rhai(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let res = env
        .engine
        .eval_expression::<Dynamic>(args.join(" ").as_str())
        .map_err(|err| {
            log::error!("Failed to eval: {:?}", err);
            "Failed to eval.".to_string()
        })?;

    Ok(Some(res.to_string()))
}

fn run_rhai(args: &[&str], env: &mut Env) -> Result<Option<String>, String> {
    let script = args[0];

    let ast = if let Some(ast) = env.ast.get(script) {
        ast
    } else {
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
            .map_err(|err| format!("Failed to compile {}: {}", script, err))?;
        ast.set_source(script);

        env.ast.insert(script.to_string(), ast);
        env.ast.get(script).unwrap()
    };

    let mut scope = Scope::new();
    scope.push_constant("World", WorldEnv::new(env.world));
    env.engine
        .consume_ast_with_scope(&mut scope, ast)
        .map_err(|err| format!("Error running {}: {}", script, err))?;

    Ok(None)
}

impl WorldEnv {
    fn new(world: &mut hecs::World) -> Self {
        WorldEnv { inner: world }
    }

    fn len(&self) -> i32 {
        unsafe { self.inner.as_ref() }.unwrap().len() as i32
    }
}

#[export_module]
mod world_api {
    #[rhai_fn(get = "len", pure)]
    pub fn get_len(world: &mut WorldEnv) -> i32 {
        world.len()
    }
}
