
#![warn(missing_docs)]

//! myhhfhf is a minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

mod sample01;

use std::os::raw::{c_char};
use env_logger::Env;
use log::{info};
use clap::{App, Arg};
use ffi_log2::{LogParam, log_param};


#[link(name = "sample01", kind = "dylib")]
extern {
    //! CAPI methods from shared library
    // fn test();
    fn sample01_run();
    fn sample01_init_logger(filter_env_var: *const c_char, write_env_var: *const c_char);
}


#[link(name = "uservice", kind = "dylib")]
extern {
    //! CAPI methods from shared library
    // fn test();
    fn runService();
    // fn init_logger(filter_env_var: *const c_char, write_env_var: *const c_char);
    fn register_callback(callback: extern fn(i32)) -> i32;
    fn trigger_callback();
    fn uservice_init_logger_ffi(param: LogParam);
}

extern fn callback(a: i32) {
    info!("i am a log of callback from main");
    println!("I'm called from UService library with value {0}", a);
}


fn register_service() {
    info!("Registering service");

    unsafe {
        register_callback(callback);
        trigger_callback(); // Triggers the callback.
    }

    unsafe {
        sample01_run();
    }

    // register the init function to be called by uservice on start
    // register the process function to be called by uservice on process
    info!("Completed registration process");
}


pub fn main() {
    //! Initialise the shared library

    // Initialize logging in main and use it from library

    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    unsafe{
        uservice_init_logger_ffi(log_param());
    }



    let matches = App::new("k8s uService")
        .version("0.1.0")
        .author("B. Greene <BenJGreene+github@gmail.com>")
        .about("Rust uService")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .about("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .takes_value(true)
                .about("Sets the level of verbosity"),
        )
        .subcommand(App::new("validate").about("Validate input yaml"))
        .subcommand(App::new("start").about("Start service"))
        .subcommand(App::new("version").about("Version info"))
        .get_matches();


    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
        panic!("Config loading not implemented yet");
    }
    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let verbose = matches.occurrences_of("v");

    if verbose > 0 {
        println!("Verbosity set to: {}", verbose);
    }

    match matches.subcommand() {
        Some(("version", _version_matches)) => {
            const NAME: &str = env!("CARGO_PKG_NAME");
            println!("Name: {}", NAME);
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("Version: {}", VERSION);
        }
        Some(("validate", validate_matches)) => {
            println!("parse and validate {:?}", validate_matches);
            panic!("validate not implemented yet");
        }
        Some(("start", _start_matches)) => {
            info!("Calling start");

            register_service();

            unsafe{
                runService();
            }
            info!("I AM DONE");
            // start(&UServiceConfig {
            //     name: String::from("simple"),
            // });
        }
        None => println!("No command provided"),
        _ => unreachable!(),

    }

}