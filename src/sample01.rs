use env_logger::Env;
use log::info;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::process;

/// Initialize the logger
///
/// The logger env_logger is built into the so and is initialised using this function. BUT this is not quite right.
/// The logger should not have to be implemented inside the so. It should be possible to implement the logger in the exe and not the so.
/// The so role is to use the log methods and not have to implement the log backend.
///
/// ```
/// use std::ffi::{CString};
/// let log_env = CString::new("USERVICE_LOG_LEVEL").expect("CString::new failed");
/// let write_env = CString::new("USERVICE_WRITE_STYLE").expect("CString::new failed");
///
/// unsafe{uservice::init_logger(log_env.as_ptr(), write_env.as_ptr());}
/// ```
#[no_mangle]
pub extern "C" fn init_service_logger(filter_c_str: *const c_char, write_c_str: *const c_char) {
    if filter_c_str.is_null() {
        panic!("Unable to read filter env var");
    }
    if write_c_str.is_null() {
        panic!("Unable to read write env var");
    }

    let filter_env = unsafe { CStr::from_ptr(filter_c_str) }
        .to_str()
        .expect("convert name to str");
    let write_env = unsafe { CStr::from_ptr(write_c_str) }
        .to_str()
        .expect("convert name to str");

    let log_level = Env::new()
        .filter_or(filter_env, "info")
        .write_style_or(write_env, "always");
    env_logger::Builder::from_env(log_level).init();
}

/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::runService();
/// ```
#[no_mangle]
pub extern "C" fn init_service() {
    info!("Initializing the service with PID: {}", process::id());

    info!("Service initialized");
}

/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::runService();
/// ```
#[no_mangle]
pub extern "C" fn process() {
    info!("Processing the service with PID: {}", process::id());

    info!("end of processing");
}
