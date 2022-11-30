use crate::ast::node::Expression;
use crate::parser::token::Token;

#[derive(Debug,PartialEq)]
pub struct InfixExpression {
    pub left:Expression,
    pub right:Expression,
    pub op_symbol:Token
}

impl InfixExpression {
    pub fn new(left:Expression) -> InfixExpression {
        Self {
            left,
            right:Expression::Unknown,
            op_symbol:Token::Unknown,
        }
    }
}
