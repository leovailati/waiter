
// TODO: add to tiny branch
//#![feature(alloc_system)]
//extern crate alloc_system;

//extern crate rouille;
extern crate hyper;
extern crate clap;

//use rouille::{Request, Response, Server};
use hyper::server::{Server, Request, Response};
use clap::{Arg, App};

use std::net::{IpAddr, ToSocketAddrs};
use std::sync::{Mutex, Arc};
use std::fs::File;
use std::path::Path;
use std::string::ToString;
use std::process;
use std::error::Error;

mod local_ip;

const DEFAULT_PORT: u16 = 8000;


fn file_handler(req: Request, res: Response) {
    
}

fn main() {
    let port_str = &DEFAULT_PORT.to_string();

    let matches = App::new("Waiter")
        .version("0.1")
        .author("Leo Vailati <leovailati@gmail.com>")
        .about("Serves static files over HTTP.")
        .arg(Arg::with_name("address")
                 .short("a")
                 .long("address")
                 .value_name("ADDRESS")
                 .help("Sets address for server. If not provided, waiter will try to figure out by itself what the local address sould be.")
                 .takes_value(true))
        .arg(Arg::with_name("port")
                 .short("p")
                 .long("--port")
                 .value_name("PORT")
                 .help("Sets TCP port for server.")
                 .default_value(port_str)
                 .takes_value(true))
        .arg(Arg::with_name("count")
                 .short("c")
                 .long("--count")
                 .value_name("COUNT")
                 .help("Server will exit after <COUNT> succesful requests.")
                 .takes_value(true))
        .arg(Arg::with_name("file")
                 .help("File to be served")
                 .required(true)
                 .index(1))
        .get_matches();

    let pathstr = matches.value_of("file").unwrap(); // unwrapping because "file" is a required parameter
    let path = Path::new(pathstr).to_owned();

    //let count_mutex = Mutex::new(count);

    let port = match matches.value_of("port") {
        None => DEFAULT_PORT,
        Some(s) => num_arg_validator::<u16>(s),
    };

    let count = match matches.value_of("count") {
        None => None,
        Some(s) => Some(num_arg_validator::<usize>(s)),
    };

    let server = match matches.value_of("address") {
        Some(val) => Server::http((val, port)),
        None => {
            let ip =
                local_ip::local_ip().expect("Unable to automatically determine local IP address.");
            Server::http((ip, port))
        }
    };

    server.

    //println!("🍔  Now serving {} on http://{}.",
    //         pathstr,
    //         server.local_addr().unwrap());

    /*
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


    server.run();*/

    println!("{:?}", local_ip::local_ip());
}

use std::str::FromStr;
use std::num::ParseIntError;

fn num_arg_validator<N: FromStr<Err = ParseIntError>>(num_str: &str) -> N {
    match num_str.parse::<N>() {
        Ok(x) => x,

        Err(e) => {
            let desc = e.description().to_owned();
            let err = clap::Error::value_validation_auto(desc);
            graciously_exit(&err)
        }
    }
}

use std::fmt::Display;

fn graciously_exit<T: Display>(msg: &T) -> ! {
    println!("{}", msg);
    process::exit(0)
}
