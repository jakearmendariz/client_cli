use std::net::TcpStream;
use std::str;
use std::env::args;
use std::process;

// http = 80, https = 443, ftp = 21, etc.) unless the port number is specifically typed in the URL (for example "http://www.simpledns.com:5000" = port 5000).

fn main() {
    let flag = args().nth(1).expect("please provide an argument, --help for help");
    let port = 80;
    let mut host = "https://my-worker.jakearmendariz.workers.dev/";
    match &flag[..] {
        "--url" => {
            host = &args().nth(2).expect("--url missing url argument").to_string();
            println!("using url: {:?}",host);
        },
        "--help" => {
            println!("client tool. Run with client --url <url>");
            println!("client --profile runs diagnostics");
            process::exit(0x0100);
        },
        "--profile" => {
            println!("Begining diagnostics");
            process::exit(0x0100);
        },
        _ => {
            println!("client tool. Run with client --url <url>");
            println!("client --profile runs diagnostics");
            process::exit(0x0100);
        }
    }

    // HTTP endpoint
    
    let path = "/";

    // Attempt to convert host to IP address
    //   let ip_lookup = get_host_ip(host).unwrap();
    // let ip_lookup: Vec<std::net::IpAddr> = host.to_socket_addrs()
    //     .expect("Unable to resolve domain")
    //     .collect();
    // println!("IP: {:?}", ip_lookup);\
    // let server_details = "127.0.0.1:80";
    // let server: Vec<_> = server_details
    //     .to_socket_addrs()
    //     .expect("Unable to resolve domain")
    //     .collect();
    // println!("{:?}", server);


    // Open socket connection ip_lookup.join(".")
    let mut stream = TcpStream::connect(format!("{}:{}",host, port))
                        .expect("Couldn't connect to the server...");

    // Format HTTP request
    let mut header = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path.clone(), host.clone());
    println!("{:?}", header);
    stream.write(header.as_bytes()).expect("Couldn't write to the server...");

    let mut buffer = vec![0 as u8; 4090]; // using 50 byte buffer
    // Make request and return response as string
    let bytes_read = stream.read(&mut buffer).expect("Couldn't read from the server...");
    println!("{:?}", str::from_utf8(&buffer[0..bytes_read]).unwrap());
}