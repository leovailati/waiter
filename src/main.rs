extern crate rouille;
extern crate clap;
extern crate local_ip;

use rouille::{Request, Response};
use clap::{Arg, App};

use std::net::IpAddr;
use std::fs::File;
use std::path::Path;
use std::string::ToString;

const DEFAULT_PORT: u16 = 8000;

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
        .arg(Arg::with_name("single")
                 .long("--single")
                 .help("Server will exit after first succesfull request"))
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

    println!("🍔  Now serving {} on http://{}:{}.", pathstr, addr, port);

    rouille::start_server((addr.as_ref(), port), move |req| {
        let url = req.url();
        println!("💁  Received {} request for URL {} from {}", req.method(), url, req.remote_addr());
        if url == "/" {
            println!("👍  Serving {}...", path.to_string_lossy());

            let file = File::open(&path).expect("Failed to open the given path");

            Response::from_file("application/octet-stream", file)
                .with_unique_header("content-disposition",
                                    format!("attachment; filename={}",
                                            path.file_name().unwrap().to_string_lossy()))
        } else {
            println!("👮  Responding with 404.");

            Response::empty_404()
        }

    });
}
