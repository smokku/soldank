use crate::mq;
use log::{Level, Metadata, Record};
use std::sync::{Arc, RwLock};

type Log = Arc<RwLock<Vec<(Level, f64, String)>>>;
static mut LOG: Option<Log> = None;

pub struct Logger;
static LOGGER: Logger = Logger;

static mut INNER: Option<env_logger::Logger> = None;

impl Logger {
    pub fn init() {
        unsafe {
            LOG.replace(Arc::new(RwLock::new(Vec::new())));
        }

        let mut builder = env_logger::Builder::from_env(env_logger::Env::default());
        let logger = builder.build();
        let max_level = logger.filter();
        unsafe {
            INNER.replace(logger);
        }

        if let Ok(()) = log::set_logger(&LOGGER) {
            log::set_max_level(max_level)
        }
    }

    pub fn get_log() -> Log {
        unsafe { LOG.as_ref().unwrap() }.clone()
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        unsafe { INNER.as_ref().unwrap() }.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if record.level() <= Level::Info {
            unsafe { LOG.as_ref().unwrap() }.write().unwrap().push((
                record.level(),
                mq::date::now(),
                format!("{}", record.args()),
            ));
        }

        unsafe { INNER.as_ref().unwrap() }.log(record)
    }

    fn flush(&self) {
        unsafe { INNER.as_ref().unwrap() }.flush()
    }
}
