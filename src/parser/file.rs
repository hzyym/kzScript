use std::{fs::{self}};

pub struct PaserFile {
    path:String,
    pub body:Vec<u8>
}

impl PaserFile {
    pub fn new(path:&str) -> PaserFile {
        let body = fs::read(path).unwrap();
        Self {
            path:path.to_string(),
            body:body
        }
    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}


#[test]
fn test_new_file() {
    let f = PaserFile::new("./src/script/01.kz");
    println!("{:?}",f.body)   
}