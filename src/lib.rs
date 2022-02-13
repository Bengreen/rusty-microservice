use ffi_log2::LogParam;
use log::info;
use std::fmt::Display;

// TODO: consider how to do better error handling over ffi using : https://michael-f-bryan.github.io/rust-ffi-guide/errors/return_types.html

#[repr(C)]
pub struct SoLibrary {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct SoService {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct UService {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "uservice", kind = "dylib")]
extern "C" {
    //! CAPI methods from shared library

    fn uservice_logger_init(param: LogParam);

    fn so_library_register(name: *const libc::c_char) -> *mut SoLibrary;
    fn so_library_free(library: *mut SoLibrary);

    fn so_service_register(library: *mut SoLibrary) -> *mut SoService;
    fn so_service_free(service: *mut SoService);

    fn so_service_logger_init(service: *mut SoService, param: LogParam);
    fn so_service_init(service: *mut SoService, param: i32) -> i32;
    fn so_service_process(service: *mut SoService, param: i32) -> i32;


    fn uservice_init(name: *const libc::c_char) -> *mut UService;
    fn uservice_free(uservice: *mut UService);
    fn uservice_add_so(uservice: *mut UService, soservice: *mut SoService);
    fn uservice_start(service: *mut UService);

}

/**
 * Register logging for uservice
 */
pub fn uservice_logger_init_ffi(param: LogParam) {
    unsafe { uservice_logger_init(param) };
}

/**
Register the so library with the uservice library

Convert the string name of the library into a safe form to send over ffi interface
*/
pub fn so_library_register_ffi<S: Into<String>>(
    library_name: S,
) -> Result<*mut SoLibrary, std::ffi::NulError>
where
    S: Display,
{
    info!("Registering so library: {}", &library_name);
    let c_library_name = std::ffi::CString::new(library_name.into())?;
    let ben = unsafe {
        // Call actual FFI interface
        so_library_register(c_library_name.as_ptr())
    };

    Ok(ben)
    // The lifetime of c_err continues until here
}

/**
 * Deregister the shared library
 */
pub fn so_library_free_ffi(library: *mut SoLibrary) {
    unsafe {
        so_library_free(library);
    }
}

/**
 * Create the function set of registered service object
 */
pub fn so_service_register_ffi(
    library: *mut SoLibrary,
) -> Result<*mut SoService, std::ffi::NulError> {
    let ben = unsafe { so_service_register(library) };

    Ok(ben)
}

/**
 * Deregister the shared library service
 */
pub fn so_service_free_ffi(service: *mut SoService) {
    unsafe {
        so_service_free(service);
    }
}

/**
 * Register logger for loaded service
 */
pub fn so_service_logger_init_ffi(service: *mut SoService, param: LogParam) {
    unsafe {
        so_service_logger_init(service, param);
    }
}

/**
 * Register logger for loaded service
 */
pub fn so_service_init_ffi(service: *mut SoService, param: i32) -> i32 {
    unsafe { so_service_init(service, param) }
}

/**
 * Register logger for loaded service
 */
pub fn so_service_process_ffi(service: *mut SoService, param: i32) -> i32 {
    unsafe { so_service_process(service, param) }
}


pub fn uservice_init_ffi<S: Into<String>>(
    name: S,
) -> Result<*mut UService, std::ffi::NulError>
where
    S: Display,
{
    info!("Registering uservice: {}", &name);
    let c_name = std::ffi::CString::new(name.into())?;
    let uservice = unsafe {
        // Call actual FFI interface
        uservice_init(c_name.as_ptr())
    };

    Ok(uservice)
    // The lifetime of c_err continues until here
}


pub fn uservice_free_ffi(uservice: *mut UService) {
    unsafe {
        uservice_free(uservice);
    }
}

/** Add soservice to uservice
 *
 */
pub fn uservice_add_so_ffi(uservice: *mut UService, soservice: *mut SoService) {
    unsafe {
        uservice_add_so(uservice, soservice);
    }
}

/** Start the service passing in the SO service
 *
 */
pub fn uservice_start_ffi(service: *mut UService) {
    unsafe {
        uservice_start(service);
    }
}
