use std::io::prelude::*;
use std::net::TcpStream;
use std::str;
use std::env::args;
use std::process;
use std::io;
use std::thread;
use std::sync::Arc;
use std::collections::HashSet;
use std::sync::Mutex;
use stopwatch::{Stopwatch};
use openssl::ssl::{SslMethod, SslConnector};
use url::{Url};
use std::time::Duration;


// fn connect(host:String, port:u16) -> TcpStream {
//     let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
//     let stream = TcpStream::connect(format!("{}:{}",host,port)).unwrap();
//     stream.set_read_timeout(Some(Duration::from_millis(100))).expect("set_read_timeout call failed");
//     return connector.connect(host, stream).unwrap();
// }

fn request_profile() -> Result<usize,io::Error> {
    let url = "my-worker.jakearmendariz.workers.dev";
    let port = 443;
    let path = "/";

    // Open tcp socket connection. I used a ssl library I hope that is fine, very low on time
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect(format!("{}:{}",url,port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(100))).expect("set_read_timeout call failed");
    let mut stream = connector.connect(url, stream).unwrap();

    // Format HTTP request
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", path.clone(), url.clone());
    stream.write(header.as_bytes())?;

    let mut buffer = vec![0 as u8; 4096]; // using 4096 byte buffer (always large enough to read the page from profile)
    // Make request and count # of bytes
    let bytes_read = stream.read(&mut buffer)?;
    return Ok(bytes_read);
}

fn diagnostics_on_profile(trials:u16) {
    let successful = Arc::new(Mutex::new(0));
    let total = Arc::new(Mutex::new(std::usize::MIN)); //total bytes
    let mut handles = vec![];
    let max_bytes = Arc::new(Mutex::new(std::usize::MIN));
    let min_bytes = Arc::new(Mutex::new(std::usize::MAX));
    let error_codes = Arc::new(Mutex::new(HashSet::new()));
    let times = Arc::new(Mutex::new(vec![]));

    for _ in 0..trials {
        let successful = Arc::clone(&successful);
        let total = Arc::clone(&total); 
        let max_bytes = Arc::clone(&max_bytes);
        let min_bytes = Arc::clone(&min_bytes);

        let times = Arc::clone(&times);
        let error_codes = Arc::clone(&error_codes);

        let handle = thread::spawn(move || {
            let sw = Stopwatch::start_new();
            match request_profile() {
                Ok(bytes) => {
                    let mut tms = times.lock().unwrap();
                    tms.push(sw.elapsed_ms());

                    let mut num = successful.lock().unwrap();
                    *num += 1;
                    let mut tot = total.lock().unwrap();
                    *tot += bytes;

                    let mut max = max_bytes.lock().unwrap();
                    if *max < bytes { *max = bytes }

                    let mut min = min_bytes.lock().unwrap();
                    if *min > bytes { *min = bytes }
                },
                Err(e) => {
                    let mut errors = error_codes.lock().unwrap();
                    errors.insert(e.kind());
                }
            }
        });
        handles.push(handle);
    }

    //joins the threads
    for handle in handles {
        handle.join().unwrap();
    }
    let success_count = *successful.lock().unwrap();
    let errors = &*error_codes.lock().unwrap();
    println!("Success: {}%\naveraging bytes: {}\nmin bytes: {}\nmax bytes: {}", success_count as f32*100.0/trials as f32, *total.lock().unwrap() as i64/success_count as i64, *min_bytes.lock().unwrap(), *max_bytes.lock().unwrap());
    if errors.len() > 0 {
        println!("Errors: {}", trials - success_count);
    }
    for e in errors.into_iter() {
        println!("   {:?}", e);
    }
    
    let statistics = timing_statistics(&mut *times.lock().unwrap());
    statistics.print();
}

//Results from the diagnostics
pub struct TimingStats {
    mean:f64,
    median:i64,
    max:i64,
    min:i64
}

impl TimingStats {
    pub fn print(&self) {
        println!("mean time: {}\nmedian time: {}\nmax time: {}\nmin time: {}", self.mean, self.median, self.max, self.min);
    }
}

fn timing_statistics(times:&mut Vec<i64>) -> TimingStats {
    let mean:f64 = times.iter().sum::<i64>() as f64 / times.len() as f64;
    times.sort();
    let median = times[times.len()/2];
    return TimingStats {mean, median, max:*times.last().unwrap(), min:times[0]};
}

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
                Err(e) => {
                    println!("Error parsing # of trials: {}", e);
                    process::exit(0x0100);
                }
            };
            if trials > 0 {
                diagnostics_on_profile(trials);
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

    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect(format!("{}:{}",host,port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(100))).expect("set_read_timeout call failed");
    let mut stream = connector.connect(host, stream).unwrap();
    // let mut stream = connect(host.to_string(), port);
    println!("{}:{}{}", host, port, path);

    // Format HTTP request
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", path.clone(), host.clone());
    println!("{:?}", header);
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