use log::{info};
use std::ffi::{CStr};
use std::os::raw::{c_char, c_int};
// use env_logger::Env;
use std::process;

use ffi_log2::{LogParam, init};


mod uservice;
mod k8slifecycle;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");


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



// typedef void (*rust_callback)(int32_t);
// rust_callback cb;

// pub struct MyState {
//     pub call_back: extern fn(i32) -> i32
// }

pub struct MyState<> {
    // pub call_back: Box<dyn FnMut(i32) -> i32 + 'a>
    pub call_back: Box<extern fn(i32) -> i32 >
}

static mut MYCB: Option<MyState> = None;



/// Create a call back register function
///
/// This will store the function provided, making it avalable when the callback is to be triggered
#[no_mangle]
pub extern fn register_callback(callback: extern fn(i32) -> i32) -> i32 {
    // Save callback function that has been registered so it can be called later.
    // MyState::call_back = callback;
    callback(3);

    match get_callback() {
        Some(_b) => {
            println!("CB already set");
        },
        None => {
            println!("starting to replace callback");
            info!("UService registering callback");
            unsafe {
                MYCB=Some(MyState{call_back : Box::new(callback)});
            }
            println!("have registered callback");
        }

    }
    return 1;
}

fn get_callback() -> &'static Option<MyState> {
    unsafe {
        &MYCB
    }
}

#[no_mangle]
pub extern fn trigger_callback() {
    match get_callback() {
        Some(b) => {
            println!("have been registered OK");
            let x = (*b.call_back)(12);
            println!("x = {} on callback", x);
        },
        None => panic!("not registered yet")
    }

}


#[no_mangle]
pub extern fn uservice_init_logger_ffi(param: LogParam) {
    init(param);
    info!("Logging registered for {}:{} (PID: {}) using FFI", NAME, VERSION, process::id());
}
