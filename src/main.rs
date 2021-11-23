
#![warn(missing_docs)]

//! myhhfhf is a minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

use std::os::raw::{c_char};
use env_logger::Env;
use log::{info};
use std::ffi::{CString};
use clap::{App, Arg};


#[link(name = "uservice", kind = "dylib")]
extern {
    //! CAPI methods from shared library
    // fn test();
    fn runService();
    fn init_logger(filter_env_var: *const c_char, write_env_var: *const c_char);
}


pub fn main() {
    //! Initialise the shared library

    // Initialize logging in main and use it from library

    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    let log_env = CString::new("SNAKESKIN_LOG_LEVEL").expect("CString::new failed");
    let write_env = CString::new("SNAKESKIN_WRITE_STYLE").expect("CString::new failed");
    unsafe{init_logger(log_env.as_ptr(), write_env.as_ptr());}



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