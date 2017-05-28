
// TODO: add to tiny branch
//#![feature(alloc_system)]
//extern crate alloc_system;

extern crate hyper;
extern crate clap;
extern crate pretty_bytes;

use hyper::server::Server;
use clap::{Arg, App};

use std::sync::Arc;
use std::fs::File;
use std::path::Path;
use std::process;
use std::error::Error;
use std::io::Read;
use std::ffi::OsStr;

mod local_ip;
mod dir_nav;
mod http_serv;

use http_serv::{SingleFile, FolderNav};

const DEFAULT_PORT: u16 = 8000;

pub fn file_open(path: &Path) -> (Vec<u8>, String) {
    let file_name = path.file_name()
        .map(|s| safely_unwrap_os_str(s).to_owned())
        .unwrap_or_else(|| graciously_exit("Invalid file name."));

    let mut file = File::open(path).unwrap_or_else(|e| graciously_exit(&e));

    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)
        .unwrap_or_else(|e| graciously_exit(&e));

    (file_data, file_name)
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
                 .help("Sets address for server. If not provided, waiter will try to determine what the local address should be.")
                 .takes_value(true))
        .arg(Arg::with_name("port")
                 .short("p")
                 .long("port")
                 .value_name("PORT")
                 .help("Sets TCP port for server.")
                 .default_value(port_str)
                 .takes_value(true))
        .arg(Arg::with_name("count")
                 .short("c")
                 .long("count")
                 .value_name("COUNT")
                 .help("Server will exit after <COUNT> succesful requests.")
                 .takes_value(true))
        .arg(Arg::with_name("directory")
                 .short("d")
                 .long("dir")
                 .help("Use web directory navigation."))
        .arg(Arg::with_name("path")
                 .help("Path to file or directory to be served.")
                 .required(true)
                 .index(1))
        .get_matches();

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
            let ip = local_ip::local_ip().unwrap_or_else(|_| {
                graciously_exit("Unable to automatically determine local address.");
            });
            Server::http((ip, port))
        }
    };

    let mut server = server.unwrap_or_else(|e| graciously_exit(&e));

    let local_addr = server
        .local_addr()
        .unwrap_or_else(|e| graciously_exit(&e));
    let local_addr = Arc::new(local_addr);

    let path_str = matches.value_of("path").unwrap();
    // unwrapping because "path" is a required parameter, clap will report error if not present
    let path = Path::new(path_str);

    // File or folder?
    if path.is_dir() {
        // Serving folder
        if matches.is_present("directory") {
            let folder_nav_server = FolderNav::new_dir_nav(count, path.to_owned());

            match server.handle(folder_nav_server) {
                Ok(_) => {
                    println!("🍔  Now serving {} on http://{}", path_str, local_addr);
                }
                Err(e) => graciously_exit(&e),
            };

        } else {
            graciously_exit("Use --dir to enable web directory navigation.");
        }

    } else if path.is_file() {
        // Serving file
        let (file_data, file_name) = file_open(path);

        let file_server = SingleFile::new_single_file(count, Arc::new(file_data), file_name);

        match server.handle(file_server) {
            Ok(_) => {
                println!("🍔  Now serving {} on http://{}", path_str, local_addr);
            }
            Err(e) => graciously_exit(&e),
        };

    } else {
        graciously_exit("Provided path was neither a file nor folder.");
    }
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

pub fn graciously_exit<T: Display>(msg: T) -> ! {
    println!("{}", msg);
    process::exit(0)
}

pub fn safely_unwrap_os_str(s: &OsStr) -> &str {
    s.to_str().unwrap_or("NON_UTF8_NAME")
}
