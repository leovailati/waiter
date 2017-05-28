use std::path::Path;
use pretty_bytes::converter;

use super::{graciously_exit, safely_unwrap_os_str};

pub fn get_dir_nav_html(path: &Path) -> String {
    let iter = path.read_dir()
        .unwrap_or_else(|e| graciously_exit(&e))
        .filter_map(|item| item.ok());
    /*
    for entry in .unwrap() {
        let entry = entry.unwrap();
        let m = entry.metadata().unwrap();
        println!("{:?} | is dir: {} | is file: {} | len: {}",
                 entry.file_name(),
                 m.is_dir(),
                 m.is_file(),
                 converter::convert(m.len() as f64));


    }*/

    let mut s = String::new();

    for item in iter {
        let metadata = item.metadata().unwrap();
        let name = item.file_name();
        let name_str = safely_unwrap_os_str(&name);
        let len = metadata.len();

        s.push_str("\r\n");
        if metadata.is_file() {
            s.push_str("FILE ");
            s.push_str(name_str);
            s.push(' ');
            s.push_str(&converter::convert(len as f64));
        } else if metadata.is_dir() {
            s.push_str("DIR  ");
            s.push_str(name_str);
            s.push(' ');
            s.push_str(&converter::convert(len as f64));
        }
    }
    s.push_str("\r\n");
    s

}
