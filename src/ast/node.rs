use crate::ast::for_::ForStatement;
use crate::ast::fun::{FunExpression};
use crate::ast::ifs::IfStatement;
use crate::ast::infix::InfixExpression;
use crate::ast::lets::LetStatement;
use crate::parser::token::Token;

#[derive(Debug,PartialEq)]
pub enum  Statement {
    Let(LetStatement),
    IF(IfStatement),
    Fun(FunExpression),
    Type(String,Expression),
    Invoke(Expression),
    For(ForStatement),
    Return(Expression),
    Break,
    Continue,
}
#[derive(Debug,PartialEq)]
pub enum Expression {
    Int(i32),
    String(String),
    Float(f32),
    Bool(bool),
    Infix(Box<InfixExpression>),
    Ident(String),
    Param(String,Token),
    Struct(Box<Vec<Expression>>),
    FunType(Option<Box<Vec<Expression>>>,Option<Token>),
    Basic(Token),
    ListValue(Vec<Box<Expression>>),
    Call(Box<Expression>,Vec<Box<Expression>>),
    Index(Box<Expression>,Box<Expression>),
    SelfOp(Box<Expression>,Token,bool),
    Fun(Box<FunExpression>),
    Unknown
}