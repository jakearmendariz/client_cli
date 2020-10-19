use std::io::prelude::*;
use std::net::TcpStream;
use std::str;
use std::process;
use openssl::ssl::{SslMethod, SslConnector};
use url::{Url};
use std::time::Duration;
use clap::{Arg, App};

mod lib;
mod ddos;

fn main() {
    let arguments = App::new("Rust Client")
        .author("Jake Armendariz <jakearmendariz99@gmail.com>")
        .arg(Arg::with_name("url")
            .long("url")
            .help("Sets a custom url to ping")
            .takes_value(true))
        .arg(Arg::with_name("count")
            .long("profile")
            .short("p")
            .help("Specifies to # of requests to send to https://my-worker.jakearmendariz.workers.dev/")
            .takes_value(true))
        .arg(Arg::with_name("single_count")
            .long("single")
            .short("s")
            .help("Sequentially makes <count> requests to https://my-worker.jakearmendariz.workers.dev/")
            .takes_value(true))
        .get_matches();
    

    if arguments.occurrences_of("count") > 0 {
        let trials = match arguments.value_of("count").unwrap().parse::<u16>() {
            Ok(trials) => trials,
            Err(e) => {
                println!("Error: {}", e);
                process::exit(0x0100);
            }
        };
        if arguments.occurrences_of("single_count") > 0 {
            lib::single_threaded_diagnostics(trials);
        }else {
            lib::multi_threaded_diagnostics(trials);
        }
        process::exit(0x0100);
    } else if arguments.occurrences_of("url") == 0 {
        println!("please provide an agrument: --url or --profile --help for instruction");
        process::exit(0x0100);
    }

    let url = arguments.value_of("url").unwrap();

    let parsed_url = Url::parse(&url[..]).expect("Could not parse url");
    // let path = parsed_url.path();
    let port;
    if parsed_url.scheme().eq("http") {
        port = 80;  // http
    }else {
        port = 443; // https
    }
    let host = match parsed_url.host_str() {
        Some(host) => host.to_string(),
        None => {
            println!("Could not parse host from url");
            process::exit(0x0100);
        }
    };

    ddos::attack(100,host, port);

    // let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", path.clone(), host.clone());
    // println!("header: {}", header);
    // let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    // let stream = TcpStream::connect(format!("{}:{}",host,port)).unwrap();
    // stream.set_read_timeout(Some(Duration::from_millis(1000))).expect("set_read_timeout call failed");
    // let mut stream = connector.connect(host, stream).unwrap();

    // // Send HTTP request
    // stream.write(header.as_bytes()).expect("Couldn't write to the server...");

    // let mut buffer = vec![0 as u8; 4096];
    // // Make request and return response as string
    // let mut bytes_read = 1;
    // while bytes_read > 0 {
    //     bytes_read = match stream.read(&mut buffer) {
    //         Ok(bytes) => bytes,
    //         Err(_) => {
    //             //finished reading
    //             break;
    //         }
    //     };
    //     let buffer_str = match str::from_utf8(&buffer[0..bytes_read]) {
    //         Ok(string) => string,
    //         Err(_) => {
    //             //invalid byte sequence
    //             ""
    //         }
    //     };
    //     println!("{:?}", buffer_str);
    // }
}