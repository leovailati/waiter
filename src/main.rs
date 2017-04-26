extern crate rouille;
extern crate clap;
extern crate local_ip;

use rouille::{Request, Response, Server};
use clap::{Arg, App};

use std::net::IpAddr;
use std::sync::{Mutex, Arc};
use std::fs::File;
use std::path::Path;
use std::string::ToString;
use std::process::exit;

const DEFAULT_PORT: u16 = 8000;

// Operation modes: file, navigate folder

enum Count {
    Infinite,
    Limited(usize),
}

fn main() {
    let matches = App::new("Waiter")
        .version("0.1")
        .author("Leo Vailati <leovailati@gmail.com>")
        .about("Serves static files over HTTP.")
        .arg(Arg::with_name("address")
                 .short("a")
                 .long("address")
                 .value_name("ADDRESS")
                 .help("Sets address for server")
                 .takes_value(true))
        .arg(Arg::with_name("port")
                 .short("p")
                 .long("--port")
                 .value_name("PORT")
                 .help("Sets TCP port for server (default 8000)")
                 .takes_value(true))
        .arg(Arg::with_name("count")
                 .short("c")
                 .long("--count")
                 .value_name("CAOUNT")
                 .help("Server will exit after COUNT succesful requests")
                 .takes_value(true))
        .arg(Arg::with_name("file")
                 .help("File or folder to be served")
                 .required(true)
                 .index(1))
        .get_matches();

    let addr = matches
        .value_of("address")
        .map(String::from)
        .unwrap_or_else(|| {
                            local_ip::get()
                                .expect("Unable to automatically determine local IP address.")
                                .to_string()
                        });

    let port = matches
        .value_of("port")
        .map(|v| v.parse::<u16>().expect("Invalid port number."))
        .unwrap_or(DEFAULT_PORT);

    let pathstr = matches.value_of("file").unwrap(); // unwrapping because "file" is a required parameter
    let path = Path::new(pathstr).to_owned();

    let count = matches
        .value_of("count")
        .map(|v| {
                 Count::Limited(v.parse::<usize>()
                                    .expect("Invalid count value. Must be integer > 0."))
             })
        .unwrap_or(Count::Infinite);

    let count_mutex = Mutex::new(count);

    let server = Server::new((addr.as_ref(), port), move |req| {
        let url = req.url();
        println!("💁  Received {} request for URL {} from {}",
                 req.method(),
                 url,
                 req.remote_addr());
        if url == "/" {
            if let Count::Limited(ref mut count) = *count_mutex.lock().unwrap() {
                if *count > 0 {
                    println!("👍  Serving {}...", path.to_string_lossy());
                    *count -= 1;

                    let file = File::open(&path).expect("Failed to open the given path");

                    Response::from_file("application/octet-stream", file)
                        .with_unique_header("content-disposition",
                                            format!("attachment; filename={}",
                                                    path.file_name().unwrap().to_string_lossy()))
                } else {
                    Response::text("Sorry boss, you are late to the party.\r\n")
                }
            } else {
                let file = File::open(&path).expect("Failed to open the given path");

                Response::from_file("application/octet-stream", file)
                    .with_unique_header("content-disposition",
                                        format!("attachment; filename={}",
                                                path.file_name().unwrap().to_string_lossy()))
            }

        } else {
            println!("👮  Responding with 404.");

            Response::empty_404()
        }

    })
            .unwrap();

    println!("🍔  Now serving {} on http://{}:{}.", pathstr, addr, port);
    server.run();
}
