use std::io::prelude::*;
use std::net::TcpStream;
use std::str;
use std::env::args;
use std::process;
use std::io;
use std::thread;
// use std::sync::atomic::AtomicU32;
use std::sync::Arc;
// use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use stopwatch::{Stopwatch};
use openssl::ssl::{SslMethod, SslConnector};

// http = 80, https = 443, ftp = 21, etc.) unless the port number is specifically typed in the URL (for example "http://www.simpledns.com:5000" = port 5000).
fn request_profile() -> Result<usize,io::Error> {
    let url = "my-worker.jakearmendariz.workers.dev";
    let port = 443;
    let path = "/";

    // Open tcp socket connection. I used a ssl library I hope that is fine, very low on time
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect(format!("{}:{}",url,port)).unwrap();
    let mut stream = connector.connect(url, stream).unwrap();

    // Format HTTP request
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", path.clone(), url.clone());
    stream.write(header.as_bytes())?;

    let mut buffer = vec![0 as u8; 4096]; // using 4096 byte buffer (always large enough to read the page from profile)
    // Make request and count # of bytes
    let bytes_read = stream.read(&mut buffer)?;
    return Ok(bytes_read);
}

pub struct TimingStats {
    mean:f64,
    median:i64,
    max:i64,
    min:i64
}

impl TimingStats {
    pub fn print(&self) {
        println!("mean time: {}\nmedian time: {}\nmax time: {}\nmin time: {}\n", self.mean, self.median, self.max, self.min);
    }
}

fn timing_statistics(times:&mut Vec<i64>) -> TimingStats {
    let mean:f64 = times.iter().sum::<i64>() as f64 / times.len() as f64;
    times.sort();
    let median = times[times.len()/2];
    return TimingStats {mean, median, max:*times.last().unwrap(), min:times[0]};
}


fn main() {
    let flag = args().nth(1).expect("please provide an argument, --help for help");
    let port = 80;
    // let mut host = "https://my-worker.jakearmendariz.workers.dev/";
    let url;
    match &flag[..] {
        "--url" => {
            url = args().nth(2).expect("--url missing url argument");
            println!("using url: {:?}",url);
        },
        "--help" => {
            println!("client tool. Run with client --url <url>");
            println!("client --profile runs diagnostics");
            process::exit(0x0100);
        },
        "--profile" => {
            let trials = 20;
            println!("Begining diagnostics");
            let successful = Arc::new(Mutex::new(0));
            let total = Arc::new(Mutex::new(std::usize::MIN)); //total bytes
            let mut handles = vec![];
            let max_bytes = Arc::new(Mutex::new(std::usize::MIN));
            let min_bytes = Arc::new(Mutex::new(std::usize::MAX));

            let times = Arc::new(Mutex::new(vec![]));

            for _ in 0..trials {
                let successful = Arc::clone(&successful);
                let total = Arc::clone(&total); 
                let max_bytes = Arc::clone(&max_bytes);
                let min_bytes = Arc::clone(&min_bytes);

                let times = Arc::clone(&times);
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
                            println!("Error: {}",e);
                        }
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
            let success_count = *successful.lock().unwrap();
            println!("Succesful reads: {}, averaging bytes: {}, min bytes: {}, max bytes: {}", success_count, *total.lock().unwrap() as i64/success_count as i64, *min_bytes.lock().unwrap(), *max_bytes.lock().unwrap());
            println!("Failures: {}", trials - success_count);
            let statistics = timing_statistics(&mut *times.lock().unwrap());
            statistics.print();
            process::exit(0x0100);
        },
        _ => {
            println!("client tool. Run with client --url <url>");
            println!("client --profile runs diagnostics");
            process::exit(0x0100);
        }
    }

    let path = "/";

    // Open socket connection ip_lookup.join(".")
    let mut stream = TcpStream::connect(format!("{}:{}",url,port))
                        .expect("Couldn't connect to the server...");

    // Format HTTP request
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", path.clone(), url.clone());
    println!("{:?}", header);
    stream.write(header.as_bytes()).expect("Couldn't write to the server...");

    let mut buffer = vec![0 as u8; 4096]; // using 50 byte buffer
    // Make request and return response as string
    let bytes_read = stream.read(&mut buffer).expect("Couldn't read from the server...");
    println!("{:?}", str::from_utf8(&buffer[0..bytes_read]).unwrap());
}