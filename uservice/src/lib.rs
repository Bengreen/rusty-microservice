use core::panic;
use std::any::Any;
use ffi_helpers::catch_panic;
use libloading::Library;
use log::{info, error};
use std::panic::catch_unwind;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::process;

use ffi_log2::{logger_init, LogParam};

mod ffi_service;
mod k8slifecycle;
mod picoservice;
mod uservice;

use crate::ffi_service::SoService;
use crate::uservice::UService;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");



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

    Box::into_raw(Box::new(
        unsafe { Library::new(libloading::library_filename(name_str)) }.unwrap(),
    ))
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

/** Initialise the UService
 *
 */
#[no_mangle]
pub extern "C" fn uservice_init<'a>(name: *const libc::c_char) -> *mut UService<'a> {
    // TODO: Correct this to follow the FFI safe error respose
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
pub extern "C" fn uservice_free(ptr: *mut UService) -> u32 {
    if ptr.is_null() {
        return 1;
    }
    info!("Releasing uservice");

    unsafe {
        Box::from_raw(ptr);
    }
    0
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
pub extern "C" fn uservice_start(ptr: *mut UService) -> u32 {
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
    0
}


#[no_mangle]
pub extern "C" fn uservice_stop(ptr: *mut UService) -> u32 {
    let uservice = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    info!("Uservice Stop called");

    info!("Stopping the service with PID: {}", process::id());

    uservice.stop();
    // let result = catch_unwind(|| {
    //     // start the service
    //     uservice.start();
    //     // uservice::start(&config, service);
    // });
    // match result {
    //     Ok(_) => info!("UService completed successfully"),
    //     Err(_) => error!("UService had a panic"),
    // }

    info!("UService stop called");
    0
}


/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn pservices_logger_init(ptr: *mut UService, param: LogParam) -> u32 {
    let uservice = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    // TODO: Check that param is not null
    0
    // logger_init(param);
    // info!(
    //     "Logging registered for {}:{} (PID: {}) using FFI",
    //     NAME,
    //     VERSION,
    //     process::id()
    // );
}

/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn pservices_init(ptr: *mut UService, config_yaml: *const libc::c_char) -> u32 {
    let uservice = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let config_yaml_str: &str = match unsafe { std::ffi::CStr::from_ptr(config_yaml) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for performing pservices_init: {}",
                e
            );
        }
    };
    // TODO: Check that param is not null
    0
    // logger_init(param);
    // info!(
    //     "Logging registered for {}:{} (PID: {}) using FFI",
    //     NAME,
    //     VERSION,
    //     process::id()
    // );
}

/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn pservice_register(ptr: *mut UService, name: *const libc::c_char, library: *const libc::c_char) -> u32 {
    let uservice = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for pservices_register name: {}",
                e
            );
        }
    };
    let library_str: &str = match unsafe { std::ffi::CStr::from_ptr(library) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for pservices_register library: {}",
                e
            );
        }
    };
    // TODO: Check that param is not null
    0
    // logger_init(param);
    // info!(
    //     "Logging registered for {}:{} (PID: {}) using FFI",
    //     NAME,
    //     VERSION,
    //     process::id()
    // );
}



/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn pservice_free(ptr: *mut UService, name: *const libc::c_char) -> u32 {
    let uservice = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for pservices_register name: {}",
                e
            );
        }
    };
    // TODO: Check that param is not null
    0
    // logger_init(param);
    // info!(
    //     "Logging registered for {}:{} (PID: {}) using FFI",
    //     NAME,
    //     VERSION,
    //     process::id()
    // );
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



/** Add SO to uservice
 */
#[no_mangle]
pub extern "C" fn uservice_add_so<'a>(
    uservice_ptr: *mut UService<'a>,
    name: *const libc::c_char,
    soservice_ptr: *mut SoService<'a>,
) {
    let uservice = unsafe {
        assert!(!uservice_ptr.is_null());

        &mut *uservice_ptr
    };
    let soservice = unsafe {
        assert!(!soservice_ptr.is_null());
        Box::from_raw(soservice_ptr)
        //&mut *soservice_ptr
    };
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for registering the so library with error: {}",
                e
            );
        }
    };
    info!("Adding {:?} ({}) to {:?}", soservice, name_str, uservice);

    uservice.add_soservice(name_str, soservice);
}

#[no_mangle]
pub extern "C" fn uservice_remove_so(
    uservice_ptr: *mut UService,
    name: *const libc::c_char,
) -> *mut SoService {
    let uservice = unsafe {
        assert!(!uservice_ptr.is_null());

        &mut *uservice_ptr
    };
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "FFI string conversion failed for registering the so library with error: {}",
                e
            );
        }
    };

    let soservice = uservice.remove_soservice(name_str);

    Box::into_raw(soservice)
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
#[derive(Debug)]
enum Error {
  Message(String),
   Unknown,
}

impl From<Box<dyn Any + Send + 'static>> for Error {
   fn from(other: Box<dyn Any + Send + 'static>) -> Error {
     if let Some(owned) = other.downcast_ref::<String>() {
       Error::Message(owned.clone())
     } else if let Some(owned) = other.downcast_ref::<String>() {
       Error::Message(owned.to_string())
     } else {
       Error::Unknown
     }
   }
}

#[no_mangle]
pub extern  "C" fn panic_check() -> u32 {
    println!("I am a simple example");

    let result:Result<u32,()> = catch_panic(|| {
        let something  = None;
        something.unwrap()
    });
    // let result:Result<u32, Box<_>> = catch_unwind(|| {
    //     let something  = None;
    //     something.unwrap()
    //     // panic!("Damn this is cool");
    // });
    let message = format!("{:?}", result);

    match result {
        Ok(_) => {
            info!("UService completed successfully");
            0
        },
        Err(e) => {
            error!("UService had a panic");
            1
        },
    }
}

#[cfg(test)]
mod panic_check_tests{
    use ffi_helpers::error_handling;
    use libc::{c_char, c_int};
    use log::info;

    use crate::panic_check;

    #[test]
    fn check_return() {
        println!("Running my test");

        let ret_val = panic_check();
        match ret_val {
            0 => println!("funciton retuned nicely"),
            _ => {
                println!("OOH OOHHH");
                let err_msg_length = error_handling::last_error_length();
                let mut buffer = vec![0; err_msg_length as usize];
                let bytes_written = unsafe {
                    let buf = buffer.as_mut_ptr() as *mut c_char;
                    let len = buffer.len() as c_int;

                    error_handling::error_message_utf8(buf, len)
                };
                match bytes_written {
                    -1 => panic!("Our buffer wasn't big enough!"),
                    0 => panic!("There wasn't an error message... Huh?"),
                    len if len > 0 => {
                        buffer.truncate(len as usize - 1);
                        let msg = String::from_utf8(buffer).unwrap();
                        println!("Error: {}", msg);
                    }
                    _ => unreachable!(),
                }

            }
        }
        // assert_eq!(ret_val, 0);
        println!("Test is completed");
    }
}





#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fmt::Debug};

    #[test]
    fn lifetime_validation() {
        #[derive(Debug)]
        struct TestMe {
            pub name: String,
        }

        impl TestMe {
            pub fn new(name: &str) -> TestMe {
                println!("I am creating: {}", name);
                TestMe {
                    name: name.to_string(),
                }
            }
        }

        impl Drop for TestMe {
            fn drop(&mut self) {
                println!("Dropping TestMe! {:?}", self);
            }
        }

        let ben = TestMe::new("hello");
        println!("ben obj: {:?}", ben);

        let roy = ben;
        println!("Roy obj: {:?}", roy);

        let mut dave = HashMap::new();

        dave.insert("roy", roy);

        println!("Dave Obj: {:?}", dave);
    }
}
