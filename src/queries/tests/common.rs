use std::fs::File;
use std::io::{ErrorKind, Read};

pub(super) fn load_access_token() -> String {
    let mut file = match File::open(".access_token") {
        Ok(v) => v,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => panic!("Unable to find file .access_token"),

            ErrorKind::PermissionDenied => {
                panic!("Permission denied when opening file .access_token")
            }

            _ => panic!("Unable to open file due to unexpected error {:?}", e),
        },
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (), // If read is successful, do nothing
        Err(e) => panic!("Unable to read file contents due to error {:?}", e),
    }
    contents
}