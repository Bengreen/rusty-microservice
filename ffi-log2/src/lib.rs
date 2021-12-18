#![warn(missing_docs)]

//! Work out what needs to be configured inside the DLL to enable the log forwarding.
//! Create a ffi function that enables the logging in the DLL to be configured (safely).
//! Createa function in the main that allows creating of the object that is used to configure the DLL funciton.


use log::{Level, LevelFilter, Log, Metadata, Record, RecordBuilder, SetLoggerError};


/// FFI-safe borrowed Rust &str. Can represents `Option<&str>` by setting ptr to null.
#[repr(C)]
pub struct RustStr {
    pub ptr: *const u8,
    pub len: usize,
}

/// FFI-safe Metadata
#[repr(C)]
pub struct ExternCMetadata {
    pub level: Level,
    pub target: RustStr,
}


/** LogParam is LogParam is a struct that transports the necessary objects to enable the configuration of the DLL logger.
 *
 */
#[repr(C)]
pub struct LogParam {
    /// function to check if logging is enabled
    /// todo: make a Metadata struct that is FFI safe
    pub enabled:fn(&Metadata) -> bool,
    /// Write a log record
    pub log: fn(&Record),
    /// flush the logs
    pub flush: fn(),
    /// value for the log level
    pub level: LevelFilter,
}

struct DLog;

static mut PARAM: Option<LogParam> = None;

/** init the DLL logging by passing in the references to the implemntation of the logging
 */
pub fn init(param: LogParam) {
    let level = param.level;
    unsafe {
        if PARAM.is_some() {
            eprint!("log should only init once");
            return;
        }
        PARAM.replace(param);
    }
    if let Err(err) = log::set_logger(&LOGGER).map(|_| log::set_max_level(level)) {
        eprint!("set logger failed:{}", err);
    }
}

fn param() -> &'static LogParam {
    unsafe { PARAM.as_ref().unwrap() }
}

impl Log for DLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (param().enabled)(metadata)
    }

    fn log(&self, record: &Record) {
        (param().log)(record)
    }

    fn flush(&self) {
        (param().flush)()
    }
}

static LOGGER: DLog = DLog;

fn enabled(meta: &Metadata) -> bool {
    log::logger().enabled(meta)
}

fn log(record: &Record) {
    log::logger().log(record)
}

fn flush() {
    log::logger().flush()
}

/** extract the log parameters from the existing log implementation so that they can be shared to the DLL
 */
pub fn log_param() -> LogParam {
    LogParam {
        enabled,
        log,
        flush,
        level: log::max_level(),
    }
}
