#![warn(missing_docs)]

//! myhhfhf is a minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

mod sample01;

// use std::os::raw::{c_char};
use clap::{App, Arg};
use env_logger::Env;
use ffi_log2::{log_param, LogParam};
use log::info;

#[link(name = "sample01", kind = "dylib")]
extern "C" {
    //! CAPI methods from shared library
    // fn sample01_run();
    fn sample01_init_logger_ffi(param: LogParam);
}

#[link(name = "uservice", kind = "dylib")]
extern "C" {
    //! CAPI methods from shared library
    fn uservice_init_logger_ffi(param: LogParam);
    fn serviceStart();
    fn register_service(
        init: extern "C" fn(i32) -> i32,
        process: extern "C" fn(i32) -> i32) -> i32;
}

extern "C" fn init_me(a: i32) -> i32 {
    info!("i am the init function from main");
    println!("I'm called from UService library with value {0}", a);
    12
}
extern "C" fn process_me(a: i32) -> i32 {
    info!("i am the process function from main");
    println!("I'm called from UService library with value {0}", a);
    17
}

pub fn main() {
    //! Initialise the shared library

    // Initialize logging in main and use it from library

    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();


    let matches = App::new("k8s uService")
        .version("0.1.0")
        .author("B. Greene <BenJGreene+github@gmail.com>")
        .about("Rust uService")
        .arg(
            Arg::new("library")
                .short('l')
                .long("library")
                .default_value("sample01")
                .value_name("LIBRARY")
                .help("Library to dynamically load for process functions")
                .takes_value(true),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .takes_value(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(App::new("validate").about("Validate input yaml"))
        .subcommand(App::new("start").about("Start service"))
        .subcommand(App::new("version").about("Version info"))
        .get_matches();

    let library = matches
        .value_of("library").expect("Library value configured");



    // if let Some(library) = matches.value_of("library") {
    //     println!("Loading library: {}", library);
    //     panic!("Library loading not working yet")
    // }

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

            info!("Loading library {}", library);


            // let lib =;
            let lib =  match unsafe { libloading::Library::new(library) } {
                Ok(lib) => lib,
                Err(error) => panic!("Problem opening the file: {:?}", error),
            };
            let process_func: libloading::Symbol<unsafe extern fn() -> u32> = match unsafe {lib.get(b"process")} {
                Ok(func) => func,
                Err(error) => panic!("Could not find process function in {} and had error {:?}", library, error),
            };
            // let process = match  libloading::Symbol<unsafe extern fn() -> u32> = lib.get(b"my_func");

            unsafe {
                uservice_init_logger_ffi(log_param());
                sample01_init_logger_ffi(log_param());
            }

            info!("Registering service");
            unsafe { register_service(init_me, process_me); }
            info!("Completed registration process");

            unsafe { serviceStart(); }
            info!("serviceStart competed");

            // unsafe { sample01_run(); }
            // unsafe { trigger_service(); }

            info!("Completed execution. Service Closing");
        }
        None => println!("No command provided"),
        _ => unreachable!(),
    }
}
