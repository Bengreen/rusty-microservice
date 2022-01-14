use log::{info};
use core::panic;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::process;

use ffi_log2::{init_logging, LogParam};

mod k8slifecycle;
mod uservice;
mod ffi_service;

use ffi_service::{set_service, unset_service, MyState};
use crate::uservice::KILL_SENDER;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Start the microservice and keep exe control until it is complete
///
/// Start the microservice and retain exec until the service exits.
///
/// ```
/// uservice::serviceStart();
/// ```
#[no_mangle]
pub extern "C" fn serviceStart() {
    info!("Initializing the service with PID: {}", process::id());

    uservice::start(&uservice::UServiceConfig {
        name: String::from("simple"),
    });

    info!("Closing the service");
}

#[no_mangle]
/// Stop the microservice and wait for shutdown to complete before yielding thread
///
/// Signal to the running service (probably started in a thread) that the service is to be stopped.
/// ```
/// use std::{thread, time};
/// let thandle = std::thread::spawn(move || {
///     uservice::serviceStop();
/// });
/// thread::sleep(time::Duration::from_secs(3));
/// uservice::serviceStop();
///
/// thandle.join().expect("UService thread complete");
/// ```
///
pub extern "C" fn serviceStop() {

    info!("Closing uservice");
    let kill = unsafe {KILL_SENDER.as_ref().unwrap().lock().unwrap().clone() };
    kill.blocking_send(()).expect("Send completes to async");

    println!("Stop request completed. Waiting for service halt.");
}

#[no_mangle]
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


/// Create a call back register function
///
/// This will store the function provided, making it avalable when the callback is to be triggered
#[no_mangle]
pub extern "C" fn register_service(
    init: extern "C" fn(i32) -> i32,
    process: extern "C" fn(i32) -> i32,
    ) -> i32 {
    // Save callback function that has been registered so it can be called later.
    set_service(MyState {
        init: Box::new(init),
        process: Box::new(process),
    });
    return 1;
}

/// Unregister service from exec environment.
///
/// Note this does not ensure to check if the function is currently running or that it may be running an async thread.
/// It simply disconnected the callback to stop it being called in future.
#[no_mangle]
pub extern "C" fn unregister_service()  -> i32 {
    unset_service();
    return 0;
}
/// Run the process function
///
/// Call the process function.
/// Throws a panic if the service has not been registered prior to calling this function.
#[no_mangle]
pub extern "C" fn process(a: i32) -> i32 {
    ffi_service::process(a).expect("Process was registered")
}


/// Initialise the FFI based logging for this crate
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
