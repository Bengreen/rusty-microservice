use ffi_log2::LogParam;
use libloading::{Library, Symbol};
use log::info;


pub struct SoService<'a> {
    pub(crate) init_logger: Symbol<'a, extern "C" fn(param: LogParam)>,
    pub(crate) init: Symbol<'a, extern "C" fn(i32) -> i32>,
    pub(crate) process: Symbol<'a, extern "C" fn(i32) -> i32>,
}

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

pub struct MyState {
    pub init: Box<extern "C" fn(i32) -> i32>,
    pub process: Box<extern "C" fn(i32) -> i32>,
}

static mut MY_SERVICE: Option<MyState> = None;

pub fn get_service() -> &'static Option<MyState> {
    unsafe { &MY_SERVICE }
}

pub fn _set_service(state: MyState) {
    match get_service() {
        Some(_b) => {
            info!("MY_SERVICE already set");
        }
        None => {
            info!("UService registering callback");
            unsafe {
                MY_SERVICE = Some(state);
            }
            info!("have registered callback");
        }
    }
}

pub fn _unset_service() {
    match get_service() {
        Some(_b) => {
            info!("MY_SERVICE unsetting");
            unsafe {
                MY_SERVICE = None;
            }
        }
        None => {
            info!("MY_SERVICE non set");
        }
    }
}

pub fn init(input: i32) -> Result<i32, &'static str> {
    match get_service() {
        Some(b) => {
            return Result::Ok((*b.init)(input));
        }
        None => {
            return Result::Err("Service not registered");
        }
    }
}

pub fn process(input: i32) -> Result<i32, &'static str> {
    match get_service() {
        Some(b) => {
            return Result::Ok((*b.process)(input));
        }
        None => {
            return Result::Err("Service not registered");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_init() {
        //! Try to call process when it is not registered yet. Then register it then try again then unregister and confirm it is now gone

        assert!(process(3).is_err(), "process should not be registered");

        extern "C" fn my_init(input: i32) -> i32 {
            input
        }
        extern "C" fn my_process(input: i32) -> i32 {
            input
        }

        set_service(MyState {
            init: Box::new(my_init),
            process: Box::new(my_process),
        });
        assert!(process(3).is_ok(), "process should be registered");
        unset_service();
        assert!(process(3).is_err(), "process should not be registered");
    }
}
