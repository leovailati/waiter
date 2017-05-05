
// TODO: add to tiny branch
#![feature(alloc_system)]
extern crate alloc_system;

//extern crate rouille;
extern crate hyper;
extern crate clap;

use hyper::server::{Server, Request, Response, Handler};
use clap::{Arg, App};

use std::sync::{Mutex, Arc};
use std::fs::File;
use std::path::Path;
use std::string::ToString;
use std::process;
use std::error::Error;
use std::io::Read;
use std::io::Write;

mod local_ip;

const DEFAULT_PORT: u16 = 8000;

struct FileServer {
    count: Mutex<Option<usize>>,
    file_data: Arc<Vec<u8>>,
    file_name: String,
}

use hyper::header::Headers;

fn set_headers_for_file(headers: &mut Headers, file_name: &str) {
    use hyper::header::{ContentDisposition, DispositionType, DispositionParam, Charset,
                        ContentType};
    use hyper::mime::{Mime, TopLevel, SubLevel};

    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(Charset::Iso_8859_1,
                                                    None,
                                                    file_name.to_owned().into_bytes())],
    };

    let content_type = ContentType(Mime(TopLevel::Application, SubLevel::OctetStream, vec![]));

    headers.set(content_type);
    headers.set(content_disposition);
}

impl Handler for FileServer {
    fn handle(&self, req: Request, mut res: Response) {
        use hyper::uri::RequestUri;
        use hyper::status::StatusCode;

        println!("💁  Received {} request for URL {} from {}",
                 req.method,
                 req.uri,
                 req.remote_addr);

        if req.uri == RequestUri::AbsolutePath("/".to_owned()) {
            set_headers_for_file(res.headers_mut(), &self.file_name);

            let mut res = res.start().unwrap_or_else(|e| graciously_exit(&e));

            res.write_all(self.file_data.as_slice());
            res.end();

            if let Some(ref mut c) = *self.count.lock().unwrap() {
                if *c > 0 {
                    *c -= 1;
                    println!("⬇️  Servings left: {}", *c);
                }

                if *c == 0 {
                    graciously_exit("Mission accomplished");
                }
            };

        } else {
            println!("👮  Responding with 404.");

            let mut status = res.status_mut();
            *status = StatusCode::NotFound;
        }
    }
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

    let path_str = matches.value_of("file").unwrap(); // unwrapping because "file" is a required parameter
    let path = Path::new(path_str);
    let mut file = File::open(path).unwrap_or_else(|e| graciously_exit(&e));

    let file_name = path.file_name()
        .unwrap_or_else(|| graciously_exit("Invalid file name."))
        .to_str()
        .unwrap_or_else(|| graciously_exit("Invalid file name."))
        .to_owned();

    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)
        .unwrap_or_else(|e| graciously_exit(&e));

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

    let mut server = server.unwrap_or_else(|e| graciously_exit(&e));
    let local_addr = server
        .local_addr()
        .unwrap_or_else(|e| graciously_exit(&e));

    let file_server = FileServer {
        count: Mutex::new(count),
        file_data: Arc::new(file_data),
        file_name: file_name,
    };

    match server.handle(file_server) {
        Ok(_) => {
            println!("🍔  Now serving {} on http://{}", path_str, local_addr);
        }
        Err(e) => graciously_exit(&e),
    };

    /*
    let server = Server::new((addr.as_ref(), port), move |req| {
        let url = req.url();

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

fn graciously_exit<T: Display>(msg: T) -> ! {
    println!("{}", msg);
    process::exit(0)
}
