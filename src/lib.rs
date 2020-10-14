use std::io::prelude::*;
use std::net::TcpStream;
use std::io;
// use std::thread;
use std::sync::{Arc};
use std::collections::HashSet;
use std::sync::Mutex;
use stopwatch::{Stopwatch};
use openssl::ssl::{SslMethod, SslConnector};
use std::time::Duration;
use std::io::{Error, ErrorKind};
use lazy_static::lazy_static;
use threadpool::ThreadPool;
use std::cmp;

const PROFILE_HOST:&str = "waterworksswim.com";//"my-worker.jakearmendariz.workers.dev";
const HTTPS_PORT:u16 = 443;
const PROFILE_PATH:&str = "/";
// Time to wait before coutning request as a failure (in ms)
const WAIT_TIME:i64 = 500;
const MAX_THREADS:usize = 50;

lazy_static! {
    static ref TIMES:std::sync::Arc<std::sync::Mutex<Vec<i64>>> = Arc::new(Mutex::new(vec![]));
}
//Creates a thread for every single request. A pool with limited number of requests would be better, but this works on a small scale
pub fn multi_threaded_diagnostics(trials:u16) {
    let diagnostics = Arc::new(Mutex::new(Diagnostics::default()));
    let error_codes = Arc::new(Mutex::new(HashSet::new()));
    let thread_count = cmp::min(MAX_THREADS, trials as usize);
    let pool = ThreadPool::new(thread_count);
    // let barrier = Arc::new(Barrier::new(thread_count + 1));

    for i in 0..trials {
        let diagnostics = Arc::clone(&diagnostics);
        let error_codes = Arc::clone(&error_codes);
        // let barrier = barrier.clone();

        pool.execute(move || {
            let sw = Stopwatch::start_new();
            match request_profile(i) {
                Ok(bytes) => {
                    if bytes != 0 {
                        let mut diag = diagnostics.lock().unwrap();
                        diag.times.push(sw.elapsed_ms() - WAIT_TIME);

                        diag.successful += 1;

                        diag.total += bytes;

                        if diag.max_bytes < bytes { diag.max_bytes = bytes }

                        if diag.min_bytes > bytes { diag.min_bytes = bytes }
                    }
                },
                Err(e) => {
                    let mut errors = error_codes.lock().unwrap();
                    errors.insert(e.kind());
                }
            }
            // barrier.wait();
        });
    }
    pool.join();
    assert_eq!(0, pool.active_count());
    // barrier.wait();
    let mut diag = diagnostics.lock().unwrap();
    diag.print(trials);
    let errors = &*error_codes.lock().unwrap();
    println!("Errors: {}", errors.len());
    for e in errors.into_iter() {
        println!("Error:{:?}", e);
    }
}

// Connects and sends a get request to cloudfare worker host
fn request_profile(request_id:u16) -> Result<usize,io::Error> {
    // Open tcp socket connection. I used a ssl library I hope that is fine
    let connector = match SslConnector::builder(SslMethod::tls()){
        Ok(connector) => connector.build(),
        Err(_) => {
            // let ssl_error = Error::new(ErrorKind::Other, "ssl builder error");
            // return Err(ssl_error)
            return Ok(0);
        }
    };
    let stream = match TcpStream::connect(format!("{}:{}",PROFILE_HOST,HTTPS_PORT)) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(e)
        }
    };
    stream.set_read_timeout(Some(Duration::from_millis(WAIT_TIME as u64))).expect("set_read_timeout call failed");
    let mut stream = match connector.connect(PROFILE_HOST, stream) {
        Ok(stream) => stream,
        Err(_) => {
            // let ssl_error = Error::new(ErrorKind::Other, "ssl could not connect");
            // return Err(ssl_error);
            return Ok(0);
        }
    };

    // Format HTTP request
    let header = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Keep-Alive\r\n\r\n", PROFILE_PATH.clone(), PROFILE_HOST.clone());
    

    let mut buffer = vec![0 as u8; 4096];
    // Make request and count # of bytes
    let mut total_bytes_read = 0;
    let sw = Stopwatch::start_new();
    stream.write(header.as_bytes())?;
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(bytes) => bytes,
            Err(e) => {
                //finished reading
                println!("{}:{}",request_id, sw.elapsed_ms() - WAIT_TIME);
                if total_bytes_read == 0 {
                    //timeout error
                    return Err(e);
                }
                break;
            }
        };
        total_bytes_read += bytes_read;
    }
    if total_bytes_read == 0 {
        let empty_response = Error::new(ErrorKind::Other, "Empty Response From Server");
        return Err(empty_response);
    }
    let mut times = TIMES.lock().unwrap();
    times.push(sw.elapsed_ms() - WAIT_TIME);
    return Ok(total_bytes_read);
}

//Struct keeps track of information while requesting the profile page
#[derive(Default)]
struct Diagnostics {
    pub successful:u32,
    pub total:usize,
    pub min_bytes:usize,
    pub max_bytes:usize,
    pub times:Vec<i64>,
    pub error_codes:HashSet<io::Error>
}

impl Diagnostics {
    pub fn print(&mut self, trials:u16){
        if self.successful > 0{
            println!("Success: {}%\naveraging bytes: {}\nmin bytes: {}\nmax bytes: {}", self.successful as f32*100.0/trials as f32, self.total as i64/self.successful as i64, self.min_bytes, self.max_bytes);
            self.timing_statistics();
        }else {
            println!("0 successful trials");
        }
    }

    fn timing_statistics(&mut self) {
        let times = TIMES.lock().unwrap();
        let mean:f64 = times.iter().sum::<i64>() as f64 / times.len() as f64;
        self.times.sort();
        let median = times[times.len()/2];
        println!("mean time: {}\nmedian time: {}\nmax time: {}\nmin time: {}", mean, median, times.last().unwrap(), times[0]);
    }
}


// Not needed, if you want to run every request individually
pub fn single_threaded_diagnostics(trials:u16) {
    let mut diagnostics = Diagnostics::default();
    let mut error_codes = HashSet::new();
    for i in 0..trials {
        let sw = Stopwatch::start_new();
        match request_profile(i) {
            Ok(bytes) => {
                diagnostics.successful += 1;

                diagnostics.times.push(sw.elapsed_ms()- WAIT_TIME);

                diagnostics.total += bytes;

                if diagnostics.max_bytes < bytes { diagnostics.max_bytes = bytes }

                if diagnostics.min_bytes > bytes { diagnostics.min_bytes = bytes }
            },
            Err(e) => {
                error_codes.insert(e.kind());
            }
        }
    }
    diagnostics.print(trials);
    println!("Errors: {}", error_codes.len());
    for e in error_codes.into_iter() {
        println!("Error:{:?}", e);
    }
}

