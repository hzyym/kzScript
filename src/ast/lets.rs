
use crate::ast::node::{Expression};
use crate::parser::token::Token;

#[derive(Debug,PartialEq)]
pub struct LetStatement {
     pub token_type:Token,
     pub ident:String,
     pub exp:Option<Expression>,
}
impl LetStatement {
    pub fn new(ident:String) -> LetStatement {
        Self {
            token_type: Token::Unknown,
            ident,
            exp: None
        }
    }
}
