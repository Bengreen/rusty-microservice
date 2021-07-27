// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's "builder pattern" method of creating arguments
// which the most flexible, but also most verbose.
use clap::{Arg, App};

mod lib;
use lib::{simple_listen, tokio_start};
mod tcpthread;
use tcpthread::thread_listen;

use lib::handle_connection;

fn main() {
    let matches = App::new("Simple CLI App")
        .version("1.0")
        .author("B. Greene <BenJGreene+github@gmail.com>")
        .about("Does awesome things")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .about("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::new("v")
            .short('v')
            .multiple_occurrences(true)
            .takes_value(true)
            .about("Sets the level of verbosity"))
        .subcommand(App::new("test")
            .about("controls testing features")
            .version("1.3")
            .author("Someone E. <someone_else@other.com>")
            .arg(Arg::new("debug")
                .short('d')
                .about("print debug information verbosely")))
        .subcommand(App::new("listen")
                .about("listen to http calls")
                .version("1.3")
                .author("B.Greene <someone_else@other.com>")
                .arg(Arg::new("thread")
                    .short('t')
                    .about("Enable threads")
                )
                .arg(Arg::new("warp")
                    .short('w')
                    .about("Enable Warp")
                )
                .arg(Arg::new("debug")
                    .short('d')
                    .about("print debug information verbosely")))
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(i) = matches.value_of("INPUT") {
        println!("Value for input: {}", i);
    }

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match matches.occurrences_of("v") {
        0 => println!("Verbose mode is off"),
        1 => println!("Verbose mode is kind of on"),
        2 => println!("Verbose mode is on"),
        _ => println!("Don't be crazy"),
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


    if let Some(ref matches) = matches.subcommand_matches("listen") {

        println!("Listening");

        if matches.is_present("warp") {
            tokio_start();
        } else {

            if matches.is_present("thread") {
                thread_listen();
            } else {
                simple_listen();
            }
        }
    }

    // Continued program logic goes here...
}
