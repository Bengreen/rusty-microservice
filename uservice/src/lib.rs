use log::{info};
use std::ffi::{CStr};
use std::os::raw::{c_char, c_int};
use env_logger::Env;
use std::process;

mod uservice;
mod k8slifecycle;

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
pub extern fn init_logger(filter_c_str: *const c_char, write_c_str: *const c_char) {
    if filter_c_str.is_null() {
        panic!("Unable to read filter env var");
    }
    if write_c_str.is_null() {
        panic!("Unable to read write env var");
    }

    let filter_env = unsafe { CStr::from_ptr(filter_c_str) }.to_str().expect("convert name to str");
    let write_env = unsafe { CStr::from_ptr(write_c_str) }.to_str().expect("convert name to str");

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
pub extern fn runService() {

    info!("Initializing the service with PID: {}", process::id());

    uservice::start(&uservice::UServiceConfig {
        name: String::from("simple"),
    });

    info!("Closing the service");
}

/// Create a health probe
///
/// Create a health probe that can be used to track health of a part of the service and used within a healthcheck to create a readiness or liveness check.
///
/// ```
/// use std::ffi::{CString};
/// let health_name = CString::new("USERVICE_LOG_LEVEL").expect("CString::new failed");
///
/// let hc = uservice::createHealthProbe(health_name.as_ptr(), 2);
/// assert_eq!(hc, 20);
/// ```
#[no_mangle]
pub extern fn createHealthProbe(name: *const c_char, margin_ms: c_int) -> c_int {
    if name.is_null() {
        panic!("Unable to read probe name");
    }

    let name_c_str = unsafe { CStr::from_ptr(name) };

    let name_str = name_c_str.to_str().expect("convert name to str");

    info!("The probe is called: {}", name_str);

    name_str.len() as i32 + margin_ms
}
