// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's "builder pattern" method of creating arguments
// which the most flexible, but also most verbose.
use clap::{App, Arg};

mod lib;
use lib::{simple_listen, tokio_start};
mod tcpthread;
use tcpthread::thread_listen;
mod k8slifecycle;
mod uservice;
use crate::uservice::{start, UServiceConfig};

fn main() {
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

    match matches.subcommand() {
        Some(("parse", validate_matches)) => {
            println!("parse and validate");

        }
        Some(("start", start_matches)) => {
            println!("Starting");

            uservice::start(&UServiceConfig{name: String::from("simple")});
        }
        None => println!("No command provided"),
        _ => unreachable!(),
    }

    // if let Some(ref matches) = matches.subcommand_matches("listen") {
    //     println!("Listening");

    //     if matches.is_present("warp") {
    //         tokio_start();
    //     } else {
    //         if matches.is_present("thread") {
    //             thread_listen();
    //         } else {
    //             simple_listen();
    //         }
    //     }
    // }

    // Continued program logic goes here...
}
