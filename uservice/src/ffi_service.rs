use std::os::raw::c_char;

use ffi_log2::LogParam;
use libloading::{Library, Symbol, Error};

/// Representation of the APIs required to load a SO for UService
///
/// Initial attempt at building this class used the libloading symbols. But after bringing through and attempting to use with warp web server I ran into an issue.
/// The warp web server assumes a `static lifetime for the web server. This contrasts with the libloading symbol with has a definitive lifetime. Therefore an object witth
/// a libloading based lifetime cannot be used by a warp object as their lifetimes are contradictory.
///
/// As an alternative we can 'eject' the lifetimes out of the symbols and take ownership of the lifetimes ourselves. This should result in a lifetime for the SoService that is not
/// contradictory with the Warp library
#[derive(Debug)]
pub struct SoService {
    pub(crate) init_logger: libloading::os::unix::Symbol<extern "C" fn(param: LogParam)>,
    pub(crate) name: libloading::os::unix::Symbol<extern "C" fn() -> *const c_char>,
    pub(crate) version: libloading::os::unix::Symbol<extern "C" fn() -> *const c_char>,
    pub(crate) init: libloading::os::unix::Symbol<extern "C" fn(i32) -> i32>,
    pub(crate) process: libloading::os::unix::Symbol<extern "C" fn(i32) -> i32>,
    library: Library,
}

/** Struct and methods to manage the Registered SO
 */
impl SoService {
    pub fn new(library: Library) -> Result<SoService, Error> {
        let so_init_logger: Symbol<extern "C" fn(param: LogParam)> = unsafe { library.get(b"init_logger") }?;
        let so_name: Symbol<extern "C" fn() -> *const c_char> = unsafe { library.get(b"name") }?;
        let so_version: Symbol<extern "C" fn() -> *const c_char> = unsafe { library.get(b"version") }?;
        let so_init: Symbol<extern "C" fn(i32) -> i32> = unsafe { library.get(b"init") }?;
        let so_process: Symbol<extern "C" fn(i32) -> i32> = unsafe { library.get(b"process") }?;

        Ok(SoService {
            init_logger: unsafe {so_init_logger.into_raw()},
            init: unsafe {so_init.into_raw()},
            process: unsafe {so_process.into_raw()},
            name: unsafe {so_name.into_raw()},
            version: unsafe {so_version.into_raw()},
            library: library,
        })
    }
}
