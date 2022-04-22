use std::os::raw::c_char;

use ffi_log2::LogParam;
use libloading::{Library, Symbol, Error};

/// Representation of the APIs required to load a SO for UService
#[derive(Debug)]
pub struct SoService<'l> {
    pub(crate) init_logger: Symbol<'l, extern "C" fn(param: LogParam)>,
    pub(crate) name: Symbol<'l, extern "C" fn() -> *const c_char>,
    pub(crate) version: Symbol<'l, extern "C" fn() -> *const c_char>,
    pub(crate) init: Symbol<'l, extern "C" fn(i32) -> i32>,
    pub(crate) process: Symbol<'l, extern "C" fn(i32) -> i32>,
}

/** Struct and methods to manage the Registered SO
 */
impl<'l> SoService<'l> {
    pub fn new(library: &'l Library) -> Result<SoService, Error> {
        let so_init_logger: Symbol<extern "C" fn(param: LogParam)> = unsafe { library.get(b"init_logger") }?;
        let so_name: Symbol<extern "C" fn() -> *const c_char> = unsafe { library.get(b"name") }?;
        let so_version: Symbol<extern "C" fn() -> *const c_char> = unsafe { library.get(b"version") }?;
        let so_init: Symbol<extern "C" fn(i32) -> i32> = unsafe { library.get(b"init") }?;
        let so_process: Symbol<extern "C" fn(i32) -> i32> = unsafe { library.get(b"process") }?;

        Ok(SoService {
            init_logger: so_init_logger,
            init: so_init,
            process: so_process,
            name: so_name,
            version: so_version,
        })
    }
}
