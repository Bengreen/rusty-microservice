use log::info;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
// use env_logger::Env;
use std::process;

use ffi_log2::{init_logging, LogParam};

mod k8slifecycle;
mod uservice;

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
pub extern "C" fn runService() {
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
pub extern "C" fn createHealthProbe(name: *const c_char, margin_ms: c_int) -> c_int {
    if name.is_null() {
        panic!("Unable to read probe name");
    }

    let name_c_str = unsafe { CStr::from_ptr(name) };

    let name_str = name_c_str.to_str().expect("convert name to str");

    info!("The probe is called: {}", name_str);

    name_str.len() as i32 + margin_ms
}


pub struct MyState {
    // pub call_back: Box<dyn FnMut(i32) -> i32 + 'a>
    pub init: Box<extern "C" fn(i32) -> i32>,
    pub process: Box<extern "C" fn(i32) -> i32>,
}

static mut MY_SERVICE: Option<MyState> = None;

/// Create a call back register function
///
/// This will store the function provided, making it avalable when the callback is to be triggered
#[no_mangle]
pub extern "C" fn register_service(
    init: extern "C" fn(i32) -> i32,
    process: extern "C" fn(i32) -> i32,
    ) -> i32 {
    // Save callback function that has been registered so it can be called later.

    match get_service() {
        Some(_b) => {
            info!("MY_SERVICE already set");
        }
        None => {
            info!("UService registering callback");
            unsafe {
                MY_SERVICE = Some(MyState {
                    init: Box::new(init),
                    process: Box::new(process),
                });
            }
            info!("have registered callback");
        }
    }
    return 1;
}

fn get_service() -> &'static Option<MyState> {
    unsafe { &MY_SERVICE }
}

#[no_mangle]
pub extern "C" fn trigger_service() {
    match get_service() {
        Some(b) => {
            info!("have been registered OK");
            let x = (*b.init)(12);
            info!("x = {} on init", x);
            let x = (*b.process)(12);
            info!("x = {} on process", x);
        }
        None => panic!("not registered yet"),
    }
}

#[no_mangle]
pub extern "C" fn uservice_init_logger_ffi(param: LogParam) {
    init_logging(param);
    info!(
        "Logging registered for {}:{} (PID: {}) using FFI",
        NAME,
        VERSION,
        process::id()
    );
}
