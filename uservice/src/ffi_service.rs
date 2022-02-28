use ffi_log2::LogParam;
use libloading::{Library, Symbol};


#[derive(Debug)]
pub struct SoService<'a> {
    pub(crate) init_logger: Symbol<'a, extern "C" fn(param: LogParam)>,
    pub(crate) init: Symbol<'a, extern "C" fn(i32) -> i32>,
    pub(crate) process: Symbol<'a, extern "C" fn(i32) -> i32>,
}


/** Struct and methods to manage the Registered SO
 *
 */
impl SoService<'_> {
    pub fn new<'a>(library: &'a Library) -> SoService {
        let so_init_logger: Symbol<extern "C" fn(param: LogParam)> =
            match unsafe { library.get(b"init_logger") } {
                Ok(func) => func,
                Err(error) => panic!("Could not find init_logger and had error {:?}", error),
            };

        let so_process: Symbol<extern "C" fn(i32) -> i32> = match unsafe { library.get(b"process") }
        {
            Ok(func) => func,
            Err(error) => panic!("Could not find process function and had error {:?}", error),
        };

        let so_init: Symbol<extern "C" fn(i32) -> i32> = match unsafe { library.get(b"init") } {
            Ok(func) => func,
            Err(error) => panic!("Could not find init function and had error {:?}", error),
        };

        SoService {
            init_logger: so_init_logger,
            init: so_init,
            process: so_process,
        }
    }
}
