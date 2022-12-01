use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::parser::token::Token;


pub struct KzError {
    path:String,
    line:i32,
    index:i32,
    err:KzErr
}
impl KzError {
    pub fn new(path:&str,line:i32,index:i32,err:KzErr) -> KzError {
        Self {
            path:path.to_string(),
            line,
            index,
            err
        }
    }
    pub fn error(self) -> String {
        return format!("{}  line->{}:{}\n\t\t{}",self.path,self.line,self.index,self.err.echo())
    }
}
impl Debug for KzError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}  line->{}:{}\n\t\t{}",self.path,self.line,self.index,self.err.echo())
    }
}
impl Display for KzError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}  line->{}:{}\n\t\t{}",self.path,self.line,self.index,self.err.echo())
    }
}

impl Error for KzError{
}

pub enum KzErr {
    Program(Token),
    Type(Token),
    Expected(Token),
    ExpectedName,
    UnExpSymbol(Token),
    UnOpSymbol(Token),
    Value(String),
    ParamName,

}
impl KzErr {
    fn echo(&self) -> String {
        match self {
            KzErr::Program(tok) => format!("program unknown type error {:?}",tok),
            KzErr::Type(tok) => format!("'{:?}' type is not legal",tok),
            KzErr::Expected(tok) => format!("expected symbol '{:?}' does not exist",tok),
            KzErr::ExpectedName => "expected name is missing".to_string(),
            KzErr::UnExpSymbol(tok) => format!("unknown expected symbol -> '{:?}'",tok),
            KzErr::UnOpSymbol(tok) => format!("unknown operation symbol -> '{:?}'",tok),
            KzErr::Value(s) => format!("unknown value input -> {}",s),
            KzErr::ParamName => "Parameter name is not a valid value".to_string(),
            _ => "Unable to process error".to_string()
        }
    }
}