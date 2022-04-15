#![warn(missing_docs)]

//! A minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

use clap::{App, Arg};
use env_logger::Env;

use ffi_log2::log_param;
use log::info;

use uservice_run::{
    uservice_init_ffi, uservice_logger_init_ffi,
    uservice_start_ffi, pservice_register_ffi, pservices_logger_init_ffi, pservices_init_ffi, pservice_free_ffi, uservice_free_ffi,
};



pub fn main() {
    //! Initialise the shared library
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
                .help("Library to dynamically load for process functions. This is automatically expanded to the OS specific library name")
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
        .value_of("library")
        .expect("Library value configured");

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
        panic!("Config loading not implemented yet");
    }

    let my_config="";

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

            uservice_logger_init_ffi(log_param());

            let uservice = uservice_init_ffi("pear").expect("UService did not initialise");
            info!("Initialised UService");

            pservice_register_ffi(uservice, "apple", library).expect("Load pservice library");
            info!("Service loaded");

            info!("initialising so logging");
            pservices_logger_init_ffi(uservice, log_param());

            info!("Registered logging for pservices");

            pservices_init_ffi(uservice, my_config).expect("init completes");
            info!("PServices init completed");

            uservice_start_ffi(uservice).expect("uservice init completes");
            info!("uservice completed and exited");

            // uservice_stop_ffi(uservice).expect("")  // NOT needed as already stopped
            pservice_free_ffi(uservice, "apple").expect("pservice freed");

            uservice_free_ffi(uservice).expect("uservice free");

            // so_library_free_ffi(lib);

            info!("service deregistered");
        }
        None => println!("No command provided"),
        _ => unreachable!(),
    }
}
