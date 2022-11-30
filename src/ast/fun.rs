use crate::ast::node::{Expression, Statement};
use crate::parser::token::Token;

#[derive(Debug)]
pub struct FunStatement {
    pub name:Option<Expression>,
    pub ret_type:Option<Token>,
    pub param_number:usize,
    pub param_exp:Option<Vec<Expression>>,
    pub body:Vec<Statement>
}

impl FunStatement {
    pub fn new() -> FunStatement{
        Self {
            name:None,
            ret_type:None,
            param_number:0,
            param_exp:None,
            body:vec![],
        }
    }
}