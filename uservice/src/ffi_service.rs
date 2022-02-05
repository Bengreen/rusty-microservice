

use log::info;

pub struct MyState {
    pub init: Box<extern "C" fn(i32) -> i32>,
    pub process: Box<extern "C" fn(i32) -> i32>,
}

static mut MY_SERVICE: Option<MyState> = None;

pub fn get_service() -> &'static Option<MyState> {
    unsafe { &MY_SERVICE }
}


pub fn set_service(state: MyState) {

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

pub fn unset_service() {
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

        set_service(MyState{
            init: Box::new(my_init),
            process: Box::new(my_process),
        });
        assert!(process(3).is_ok(), "process should be registered");
        unset_service();
        assert!(process(3).is_err(), "process should not be registered");
    }

}