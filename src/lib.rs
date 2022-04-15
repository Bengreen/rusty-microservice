// #![doc = include_str!("../README.md")]
//! uservice_run provides a service that creates a microservcie to allow the application of functions against a web service and incorporates a monitoring and logigng and health checks into the library.


use ffi_log2::LogParam;
use log::info;
use std::fmt::{Display};


// TODO: consider how to do better error handling over ffi using : https://michael-f-bryan.github.io/rust-ffi-guide/errors/return_types.html

/// Opaque object representing SoLibrary objects
#[repr(C)]
pub struct SoLibrary {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque object representing PService objects
#[repr(C)]
pub struct PService {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque object representing UService objects
#[repr(C)]
pub struct UService {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
///
/// Register logging for uservice
/// ```mermaid
/// sequenceDiagram
///     participant Main
///     participant UService
///     participant Sample01
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Main register library and SoService
///
///     Main->>+UService: so_library_register
///     UService->>-Main: (SoLibrary)
///
///     Main->>+UService: so_service_register_ffi(SoLibrary)
///     UService->>-Main: (SoService)
///     end
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Initialise logging into SoService library
///     Main->>UService: so_service_logger_init_ffi(SoService, logconfig)
///     end
///
///
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Load UService by name
///     Main->>+UService: uservice_register_ffi(SoService, name)
///     UService->>+Sample01: so_library_register(name)
///     Sample01->>-UService: (SoLibrary)
///     UService->>+Sample01: so_uservice_register_ffi(Solibrary)
///     Sample01->>-UService: (UService)
///     UService->>-Main: (UService)
///     end
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: SoService init logging
///     Main->>UService: uservice_logger_init_ffi(SoService, name, UService, logconfig)
///     UService->>Sample01: uservice_logger_init_ffi(UService, logconfig)
///     end
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Start UService
///     Main->>+UService: uservice_start(SoService, name, UService)
///     UService->>+Sample01: uservice_start(UService)
///     Sample01->>-UService: (int)
///     UService->>-Main: (int)
///     end
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Stop UService
///     Main->>+UService: uservice_stop(SoService, name, UService)
///     UService->>+Sample01: uservice_stop(UService)
///     Sample01->>-UService: (int)
///     UService->>-Main: (int)
///     end
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: DeAllocate SoServices
///     Main->>+UService: uservice_deregister(SoService, name, UService)
///     UService->>-Main: (int)
///     end
///
///
///
///
///
///  ```
/// Implement function on UService library so that they are re-usable across FFI interfaces and available to all languages
#[link(name = "uservice", kind = "dylib")]
extern "C" {
    //! CAPI methods from shared library

    /// Register SO library returning handle to it
    fn so_library_register(name: *const libc::c_char) -> *mut SoLibrary;
    /// Release SO library from handle
    fn so_library_free(library: *mut SoLibrary);
    /// Configure logging for UService
    fn uservice_logger_init(param: LogParam);

    /// Init a UService and return the reference to the UService object
    fn uservice_init(config_yaml: *const libc::c_char) -> *mut UService;
    /// Free an UService
    fn uservice_free(uservice: *mut UService)->u32;
    /// Start uservice including enclosed PServices
    fn uservice_start(service: *mut UService)->u32;
    /// Stop uservice including enclosed PServices
    fn uservice_stop(service: *mut UService)->u32;
    /// Configure logging for all PServices
    fn pservices_logger_init(uservice: *mut UService, param: LogParam) -> u32;
    fn pservices_init(uservice: *mut UService, config_yaml: *const libc::c_char) -> u32;

    /// add named pservice to uservice
    fn pservice_register(
        uservice: *mut UService,
        name: *const libc::c_char,
        library: *const libc::c_char,
    ) -> u32;
    fn pservice_free(
        uservice: *mut UService,
        name: *const libc::c_char
    ) -> u32;

}

/**
Register the so library and return its reference

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

    Ok(unsafe {
        // Call actual FFI interface
        so_library_register(c_library_name.as_ptr())
    })
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

/// Initialise logging
pub fn uservice_logger_init_ffi(param: LogParam) {
    unsafe { uservice_logger_init(param) };
}

/**
 * Create a UService instance
 */
pub fn uservice_init_ffi<S: Into<String>>(name: S) -> Result<*mut UService, std::ffi::NulError>
where S: Display
{
    info!("Registering so library: {}", &name);
    let c_name = std::ffi::CString::new(name.into())?;

    Ok(unsafe { uservice_init(c_name.as_ptr()) })
}

/**
 * Deregister the shared library service
 */
pub fn uservice_free_ffi(service: *mut UService) -> Result<(), std::ffi::NulError>  {
    unsafe {
        uservice_free(service);
    }
    Ok(())
}

/**
 * Start UService
 */
pub fn uservice_start_ffi(service: *mut UService) -> Result<(), std::ffi::NulError> {
    unsafe {
        uservice_start(service);
    }
    Ok(())
}

/**
 * Stop UService
 */
pub fn uservice_stop_ffi(service: *mut UService) {
    unsafe {
        uservice_stop(service);
    }
}

/// Initialise logging
pub fn pservices_logger_init_ffi(service: *mut UService, param: LogParam) {
    unsafe { pservices_logger_init(service, param) };
}

pub fn pservices_init_ffi<S: Into<String>>(service: *mut UService, config_yaml: S) -> Result<(), std::ffi::NulError>
where
    S: Display
{
    let c_config_yaml = std::ffi::CString::new(config_yaml.into())?;
    unsafe { pservices_init(service, c_config_yaml.as_ptr()) };

    Ok(())
}


// Error example derrived from : https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/wrap_error.html



// #[derive(Debug)]
// enum UServiceError {
//     FFICall,
//     // We will defer to the parse error implementation for their error.
//     // Supplying extra info requires adding more data to the type.
//     Parse(std::ffi::NulError),
// }

// impl Display for UServiceError {
//     fn fmt (&self, f: &mut Formatter) -> std::fmt::Result{
//         match *self {
//             UServiceError::FFICall =>
//                 write!(f, "FFI Call failed to get a valid reply"),
//             // The wrapped error contains additional information and is available
//             // via the source() method.
//             UServiceError::Parse(..) =>
//                 write!(f, "Parse error"),
//         }
//     }
// }




/// Register pservice by name
pub fn pservice_register_ffi<S: Into<String>>(service: *mut UService, name: S, library_name: S) -> Result<(), std::ffi::NulError>
where
    S: Display
{
    info!("Registering pservice: {} as {}", &library_name, &name);
    let c_name = std::ffi::CString::new(name.into())?;
    let c_library_name = std::ffi::CString::new(library_name.into())?;

    Ok(unsafe {
        pservice_register(service, c_name.as_ptr(),c_library_name.as_ptr());
    })
}

/// Free pservice by name
pub fn pservice_free_ffi<S: Into<String>>(service: *mut UService, library_name: S) -> Result<(), std::ffi::NulError>
where
    S: Display
{
    info!("Registering pservice: {}", &library_name);
    let c_library_name = std::ffi::CString::new(library_name.into())?;

    let _ret_val = unsafe {
        pservice_free(service, c_library_name.as_ptr());
    };
    // if retVal!=0 {
    //     return Err(UServiceError::FFICall)
    // }

    Ok(())
}
