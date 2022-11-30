use crate::ast::fun::FunStatement;
use crate::ast::ifs::IfStatement;
use crate::ast::infix::InfixExpression;
use crate::ast::lets::LetStatement;
use crate::parser::token::Token;

#[derive(Debug)]
pub enum  Statement {
    Let(LetStatement),
    IF(IfStatement),
    Fun(FunStatement),
    LetFun(String,FunStatement),
    Type(String,Expression),
    Invoke(Expression)
}
#[derive(Debug,PartialEq)]
pub enum Expression {
    Int(i32),
    String(String),
    Float(f32),
    Bool(bool),
    Array(Token,Box<Vec<Expression>>),
    Infix(Box<InfixExpression>),
    Ident(String),
    Param(String,Token),
    Struct(Box<Vec<Expression>>),
    Fun(Option<Box<Vec<Expression>>>,Option<Token>),
    Basic(Token),
    ListValue(Vec<Box<Expression>>),
    Call(Box<Expression>,Vec<Box<Expression>>),
    Index(Box<Expression>,Box<Expression>),
    Unknown
}