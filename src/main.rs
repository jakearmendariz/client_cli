use std::io::prelude::*;
use std::net::TcpStream;
use std::str;
use std::env::args;
use std::process;
use openssl::ssl::{SslMethod, SslConnector};
use url::{Url};
use std::time::Duration;


mod lib;

fn print_help() {
    println!("client tool. Run with client --url <url>");
    println!("client --profile runs diagnostics on my-worker.jakearmendariz.workers.dev");
}

fn main() {
    let flag = args().nth(1).expect("please provide an argument, --help for help");
    let url;
    match &flag[..] {
        "--url" => {
            url = args().nth(2).expect("--url missing url argument");
            println!("using url: {:?}",url);
        },
        "--help" => {
            print_help();
            process::exit(0x0100);
        },
        "--profile" => {
            let trials = match args().nth(2).expect("--profile missing integer number of requests argument").parse::<u16>() {
                Ok(trials) => trials,
                Err(_) => {
                    if args().nth(2).expect("--profile missing integer number of requests argument") == "--x" {
                        let single_trials = match args().nth(3).expect("--x missing integer number of requests argument").parse::<u16>() {
                            Ok(trials) => trials,
                            Err(e) => {
                                println!("Error parsing # of trials: {}", e);
                                process::exit(0x0100);
                            }
                        };
                        lib::single_threaded_diagnostics(single_trials);
                    }
                    process::exit(0x0100);
                }
            };
            if trials > 0 {
                lib::multi_threaded_diagnostics(trials);
            }
            process::exit(0x0100);
        },
        _ => {
            print_help();
            process::exit(0x0100);
        }
    }

    let parsed_url = Url::parse(&url[..]).expect("Could not parse url");
    let path = "/";
    let port;
    if parsed_url.scheme().eq("http") {
        port = 80;  // http
    }else {
        port = 443; // https
    }
    let host = match parsed_url.host_str() {
        Some(host) => host,
        None => {
            println!("Could not parse host from url");
            process::exit(0x0100);
        }
    };
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", path.clone(), host.clone());

    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect(format!("{}:{}",host,port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(100))).expect("set_read_timeout call failed");
    let mut stream = connector.connect(host, stream).unwrap();

    // Send HTTP request
    stream.write(header.as_bytes()).expect("Couldn't write to the server...");

    let mut buffer = vec![0 as u8; 4096];
    // Make request and return response as string
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(bytes) => bytes,
            Err(_) => {
                //finished reading
                break;
            }
        };
        let buffer_str = str::from_utf8(&buffer[0..bytes_read]).unwrap();
        println!("{:?}", buffer_str);
    }
}