#![warn(missing_docs)]

//! myhhfhf is a minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

use clap::{App, Arg};
use env_logger::Env;

use ffi_log2::log_param;
use log::info;

use uservice_run::{
    so_library_free_ffi, so_library_register_ffi, so_service_free_ffi, so_service_init_ffi,
    so_service_logger_init_ffi, so_service_process_ffi, so_service_register_ffi,
    uservice_add_so_ffi, uservice_init_ffi, uservice_logger_init_ffi, uservice_remove_so_ffi,
    uservice_start_ffi,
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

            let lib = so_library_register_ffi(library).expect("So library loaded");
            let soservice = so_service_register_ffi(lib).expect("Load functions from so");
            info!("Service loaded");

            info!("initialising so logging");
            so_service_logger_init_ffi(soservice, log_param());

            info!("Completed registration process");

            info!("Testing specific init and process API calls for service");
            so_service_init_ffi(soservice, 21);
            so_service_process_ffi(soservice, 22);

            info!("Testing services.. Complete");

            info!("Create UService");
            let uservice = uservice_init_ffi("hello").expect("Initialise uservice");
            uservice_add_so_ffi(uservice, "hello", soservice).ok();

            info!("Starting Service");
            uservice_start_ffi(uservice);
            info!("uservice exited");

            uservice_remove_so_ffi(uservice, "hello").ok();


            so_service_free_ffi(soservice);

            so_library_free_ffi(lib);

            info!("service deregistered");
        }
        None => println!("No command provided"),
        _ => unreachable!(),
    }
}
