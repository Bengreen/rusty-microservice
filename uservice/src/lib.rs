use core::panic;
use std::panic::catch_unwind;
use libloading::Library;
use log::{info, error};

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::process;

use ffi_log2::{ logger_init, LogParam};

mod ffi_service;
mod k8slifecycle;
mod uservice;
mod picoservice;
pub mod simpleservice;

use crate::ffi_service::SoService;
use crate::uservice::{KILL_SENDER, UService};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn uservice_logger_init(param: LogParam) {
    logger_init(param);
    info!(
        "Logging registered for {}:{} (PID: {}) using FFI",
        NAME,
        VERSION,
        process::id()
    );
}

/// Register a shared library for by the name of the library
///
/// # Safety
///
/// It is the caller's guarantee to ensure `msg`:
///
/// - is not a null pointer
/// - points to valid, initialized data
/// - points to memory ending in a null byte
/// - won't be mutated for the duration of this function call
#[no_mangle]
pub extern "C" fn so_library_register<'a>(name: *const libc::c_char) -> *mut Library {
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for registering the so library with error: {}",
                e
            );
        }
    };
    info!("Registering library: {}", name_str);

    let library = Box::into_raw(Box::new(
        unsafe { Library::new(libloading::library_filename(name_str)) }.unwrap(),
    ));

    library
}

/**
 * Free the library
 */
#[no_mangle]
pub extern "C" fn so_library_free(ptr: *mut Library) {
    if ptr.is_null() {
        return;
    }
    info!("Releasing library");

    unsafe {
        Box::from_raw(ptr);
    }
}

/**
 * Register the so functions for the library
 */
#[no_mangle]
pub extern "C" fn so_service_register<'a>(ptr: *mut Library) -> *mut SoService<'a> {
    let library = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    info!("Registering functions");

    Box::into_raw(Box::new(SoService::new(library)))
}

/**
 * Free the service for the so library
 */
#[no_mangle]
pub extern "C" fn so_service_free(ptr: *mut SoService) {
    if ptr.is_null() {
        return;
    }
    info!("Releasing service");

    unsafe {
        Box::from_raw(ptr);
    }
}

/**
 * Call the process function
 */
#[no_mangle]
pub extern "C" fn so_service_logger_init(ptr: *mut SoService, param: LogParam) {
    let service = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    info!("init_logger called");
    (&service.init_logger)(param)
}

/**
 * Call the init function
 */
#[no_mangle]
pub extern "C" fn so_service_init(ptr: *mut SoService, param: i32) -> i32 {
    let service = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    info!("init called");
    (&service.init)(param)
}

/**
 * Call the process function
 */
#[no_mangle]
pub extern "C" fn so_service_process(ptr: *mut SoService, param: i32) -> i32 {
    let service = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    info!("process called");
    (&service.process)(param)
}




/** Initialise the UService
 *
 */
#[no_mangle]
pub extern "C" fn uservice_init(name: *const libc::c_char) -> *mut UService {
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for registering the so library with error: {}",
                e
            );
        }
    };
    info!("Registering library: {}", name_str);

    info!("Init UService");

    Box::into_raw(Box::new(UService::new(name_str)))
}

/** Free the UService
 *
 */
#[no_mangle]
pub extern "C" fn uservice_free(ptr: *mut UService) {
    if ptr.is_null() {
        return;
    }
    info!("Releasing uservice");

    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn uservice_add_so(uservice_ptr: *mut UService, soservice_ptr: *mut SoService, ) {
    let uservice = unsafe {
        assert!(!uservice_ptr.is_null());
        &mut *uservice_ptr
    };
    let soservice = unsafe {
        assert!(!soservice_ptr.is_null());
        &mut *soservice_ptr
    };
    info!("Adding {:?} to {:?}", soservice, uservice);
}

/** Start the microservice and keep exe control until it is complete
 *
 * retain exec until the service exits
 *
 * ```
 * uservice:uservice_start()
 * ```
 */
#[no_mangle]
pub extern "C" fn uservice_start(ptr: *mut UService) {
    let uservice = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    info!("Uservice Start called");

    info!("Initializing the service with PID: {}", process::id());

    uservice.start();
    // let result = catch_unwind(|| {
    //     // start the service
    //     uservice.start();
    //     // uservice::start(&config, service);
    // });
    // match result {
    //     Ok(_) => info!("UService completed successfully"),
    //     Err(_) => error!("UService had a panic"),
    // }

    info!("UService completed");
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
pub extern "C" fn uservice_stop(ptr: *mut UService) {
    info!("Closing uservice");
    let kill = unsafe { KILL_SENDER.as_ref().unwrap().lock().unwrap().clone() };
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
