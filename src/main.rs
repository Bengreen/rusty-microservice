//! rust_hello is a minimal microservice to implement a k8s uService.
//!
//! The service includes:
//!  * extensible http alive and ready checks
//!  * prometheus export via http
//!  * Extensible prometheus
//!  * SIGTERM safe shutdown
//!  * Minimal docker build
//!
//! Items still to be added:
//!  * [ ] Standardised Logging
//!
//! Optional Features:
//!  * [ ] kafka consumer/producer

#![warn(missing_docs)]

use clap::{App, Arg};
use env_logger::Env;
use log::{info};

use rustyhello::{UServiceConfig, start};

fn main() {
    //! Capture CLI definition and call appropriate actions
    let matches = App::new("K8s Rust uService")
        .version("1.0")
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
        .subcommand(App::new("dev").about("Dev service"))
        .subcommand(
            App::new("test")
                .about("controls testing features")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .about("print debug information verbosely"),
                ),
        )
        .subcommand(
            App::new("listen")
                .about("listen to http calls")
                .version("1.3")
                .author("B.Greene <someone_else@other.com>")
                .arg(Arg::new("thread").short('t').about("Enable threads"))
                .arg(Arg::new("warp").short('w').about("Enable Warp"))
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .about("print debug information verbosely"),
                ),
        )
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let verbose = matches.occurrences_of("v");

    if verbose > 0 {
        println!("Verbosity set to: {}", verbose);
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(ref matches) = matches.subcommand_matches("test") {
        // "$ myapp test" was run
        if matches.is_present("debug") {
            // "$ myapp test -d" was run
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();


    match matches.subcommand() {
        Some(("version", _version_matches)) => {
            const NAME: &str = env!("CARGO_PKG_NAME");
            println!("Name: {}", NAME);
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("Version: {}", VERSION);
        }
        Some(("parse", validate_matches)) => {
            println!("parse and validate {:?}", validate_matches);
        }
        Some(("start", _start_matches)) => {
            info!("Calling start");

            start(&UServiceConfig {
                name: String::from("simple"),
            });
        }
        Some(("dev", _dev_matches)) => {
            println!("DEV system");
        }
        None => println!("No command provided"),
        _ => unreachable!(),
    }
}
