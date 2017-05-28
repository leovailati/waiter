use hyper::server::{Request, Response, Handler};
use hyper::header::{Headers, ContentDisposition, DispositionType, DispositionParam, Charset,
                    ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::uri::RequestUri;
use hyper::status::StatusCode;
use hyper::method::Method;

use std::sync::{Mutex, Arc};
use std::path::{PathBuf, Component};
use std::io::Write;

use dir_nav::get_dir_nav_html;

use super::{graciously_exit, file_open};

fn log_validate_request(req: &Request, res: &mut Response) -> bool {
    println!("💁  Received {} request for URL {} from {}",
             req.method,
             req.uri,
             req.remote_addr);

    if req.method != Method::Get {
        println!("👮  Method is not GET.");
        let mut status = res.status_mut();
        *status = StatusCode::BadRequest;
        return false;
    }

    true
}

fn handle_increment_count(count: &Mutex<Option<usize>>) {
    if let Some(ref mut c) = *count.lock().unwrap() {
        if *c > 0 {
            *c -= 1;
            println!("Servings left: {}", *c);
        }

        if *c == 0 {
            graciously_exit("Mission accomplished");
        }
    }
}

fn set_headers_for_file(headers: &mut Headers, file_name: &str) {
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

fn set_headers_for_html(headers: &mut Headers) {
    let content_type = ContentType(Mime(TopLevel::Application, SubLevel::Html, vec![]));

    headers.set(content_type);
}

pub struct SingleFile {
    file_data: Arc<Vec<u8>>,
    file_name: String,
    count: Mutex<Option<usize>>,
}

impl SingleFile {
    pub fn new_single_file(count: Option<usize>,
                           file_data: Arc<Vec<u8>>,
                           file_name: String)
                           -> SingleFile {
        SingleFile {
            file_data: file_data,
            file_name: file_name,
            count: Mutex::new(count),
        }
    }
}

impl Handler for SingleFile {
    fn handle(&self, req: Request, mut res: Response) {
        if !log_validate_request(&req, &mut res) {
            return;
        }

        if req.uri == RequestUri::AbsolutePath("/".to_owned()) {
            set_headers_for_file(res.headers_mut(), &self.file_name);

            let mut res = res.start().unwrap_or_else(|e| graciously_exit(&e));

            res.write_all(self.file_data.as_slice())
                .and_then(|_| res.end())
                .unwrap_or_else(|e| graciously_exit(&e));
            println!("⬇️  Download!");

            handle_increment_count(&self.count);

        } else {
            println!("👮  Responding with 404.");

            let mut status = res.status_mut();
            *status = StatusCode::NotFound;
        }
    }
}

pub struct FolderNav {
    root: PathBuf,
    count: Mutex<Option<usize>>,
}

impl FolderNav {
    pub fn new_dir_nav(count: Option<usize>, root: PathBuf) -> FolderNav {
        FolderNav {
            root: root,
            count: Mutex::new(count),
        }
    }
}

impl Handler for FolderNav {
    fn handle(&self, req: Request, mut res: Response) {
        if !log_validate_request(&req, &mut res) {
            return;
        }

        let req_path_buf = PathBuf::from(format!("{}", req.uri));

        println!("x  Root: {:?}", self.root);
        println!("x  Req {:?}", PathBuf::from(format!("{}", req.uri)));
        let mut components = req_path_buf.components();

        if components.next() != Some(Component::RootDir) {
            println!("x  First compnent must be root");
            return;
        }

        if components
               .clone()
               .any(|c| match c {
                        Component::ParentDir |
                        Component::Prefix(_) |
                        Component::RootDir => true,
                        _ => false,
                    }) {
            println!("x  Bad path");
        }

        let path = components.as_path();
        let full_path = self.root.join(path);
        println!("x  Full path: {:?}", full_path);

        if full_path.is_dir() {
            set_headers_for_html(res.headers_mut());

            let html = get_dir_nav_html(&full_path);

            let mut res = res.start().unwrap_or_else(|e| graciously_exit(&e));

            res.write_all(&html.into_bytes())
                .and_then(|_| res.end())
                .unwrap_or_else(|e| graciously_exit(&e));
            println!("x  Navigation: {:?}", full_path);

        } else if full_path.is_file() {
            let (file_data, file_name) = file_open(&full_path);
            set_headers_for_file(res.headers_mut(), &file_name);

            let mut res = res.start().unwrap_or_else(|e| graciously_exit(&e));

            res.write_all(file_data.as_slice())
                .and_then(|_| res.end())
                .unwrap_or_else(|e| graciously_exit(&e));
            println!("⬇️  Download!");

            handle_increment_count(&self.count);

        } else {
            println!("👮  Responding with 404.");

            let mut status = res.status_mut();
            *status = StatusCode::NotFound;
        }
    }
}
