use std::error::Error;
use std::fmt::{Debug, Display, Formatter, write};

pub struct KzError {
    path:String,
    line:i32,
    index:i32,
    err:String
}
impl KzError {
    pub fn new(path:&str,line:i32,index:i32,err:&str) -> KzError {
        Self {
            path:path.to_string(),
            line,
            index,
            err:err.to_string()
        }
    }

    pub fn error(self) -> String {
        return format!("{}  line->{}:{}\n\t\t{}",self.path,self.line,self.index,self.err)
    }
}

impl Debug for KzError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}  line->{}:{}\n\t\t{}",self.path,self.line,self.index,self.err)
    }
}

impl Display for KzError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}  line->{}:{}\n\t\t{}",self.path,self.line,self.index,self.err)
    }
}

impl Error for KzError{

}