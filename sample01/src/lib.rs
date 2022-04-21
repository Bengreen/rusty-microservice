use log::info;
// use std::ffi::{CStr};
use ffi_log2::{logger_init, LogParam};

use std::process;
use std::ffi::CString;
use libc::c_char;
// mod simpleservice;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::runService();
/// ```
#[no_mangle]
pub extern "C" fn sample01_run() {
    info!(
        "Initializing the {} {} with PID: {}",
        NAME,
        VERSION,
        process::id()
    );

    info!("Closing {}", NAME);
}

#[no_mangle]
pub extern "C" fn init_logger(param: LogParam) {
    logger_init(param);
    info!(
        "Logging at sample01 registered for {}:{} (PID: {}) using FFI",
        NAME,
        VERSION,
        process::id()
    );
}

#[no_mangle]
pub extern "C" fn name() -> *const c_char {
    let c_str_name = CString::new(NAME).unwrap();

    c_str_name.into_raw()
}

#[no_mangle]
pub extern "C" fn version() -> *const c_char {
    let c_str_name = CString::new(VERSION).unwrap();

    c_str_name.into_raw()
}


#[no_mangle]
extern "C" fn init(a: i32) -> i32 {
    info!("i am the init function from sample01 called with {}", a);
    12
}

#[no_mangle]
extern "C" fn process(a: i32) -> i32 {
    info!("i am the process function from sample01 called with {}", a);
    17
}
