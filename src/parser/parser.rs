use std::borrow::BorrowMut;
use std::collections::BTreeMap;
use std::ops::Mul;
use crate::ast::fun::FunStatement;
use crate::ast::ifs::IfStatement;
use crate::ast::infix::InfixExpression;
use crate::ast::lets::LetStatement;
use crate::parser::file::PaserFile;
use crate::parser::{lexer, token};
use crate::parser::lexer::Lexer;
use crate::parser::token::{Operation, Token};
use crate::ast::node::{Expression, Statement};
use crate::parser::error::KzError;



pub struct Parser {
    lex:Lexer,
    current_tok:Token,
    peek_tok:Token,
}

impl Parser {
    pub fn new(l:Lexer) -> Parser {
       let mut p =  Self {
            lex:l,
            current_tok:Token::Err("parsing not started".to_string()),
            peek_tok:Token::Err("parsing not started".to_string())
        };
       p.next_token();
       p.next_token();
       p
    }
    fn current_token_is(&self, tok:Token) -> bool{
        self.current_tok == tok
    }
    fn peek_token_is(&self, tok:Token) -> bool {
        self.peek_tok == tok
    }
    fn expect_peek_token(&mut self, tok: Token) -> bool {
        if self.peek_token_is(tok) {
            self.next_token();
            return true
        }
        return false
    }
    fn expect_curr_token(&mut self, tok: Token) -> bool{
        if self.current_token_is(tok) {
            self.next_token();
            return true
        }
        return false
    }
    fn next_token(&mut self) {
        self.current_tok = self.peek_tok.clone();
        self.peek_tok = self.lex.next()
    }

    pub fn program(&mut self) -> Result<Vec<Statement>,KzError> {
        let mut v = Vec::new();
        while !self.peek_token_is(Token::Eof) && !self.peek_token_is(Token::RightCurlyBracket) {
            match self.current_tok {
                Token::Let => {
                    v.push(self.let_statement()?)
                },
                Token::If => {
                    v.push(self.if_statement()?)
                }
                Token::Fun =>{
                   v.push(Statement::Fun(self.fun_statement()?))
                },
                Token::Type => {
                    v.push(self.type_statement()?)
                },
                Token::Ident(_) => {
                    v.push(self.ident_statement()?)
                },
                Token::N => { self.next_token() },
                Token::RightCurlyBracket => { return  Ok(v)}
                _ => {
                    return Err(self.error(format!("program unknown type error {:?}",&self.current_tok).as_str()))
                }
            };

            if self.current_token_is(Token::Sem) {
                self.next_token();
            }
        }

        Ok(v)
    }
    fn ident(&self) -> Option<String>{
       match &self.peek_tok {
           Token::Ident(name) => Some(name.to_string()),
           _ => None
       }
    }
    fn let_statement(&mut self) -> Result<Statement,KzError> {
        let ident = self.ident();
        if ident != None {
           let mut let_stem = LetStatement::new(ident.unwrap());
            self.next_token();
            if self.expect_peek_token(Token::Colon) {
                self.next_token();
                if self.current_token_is(Token::LeftSquareBra) {
                    self.array_token();
                }
                if !Token::let_type(&self.current_tok) {
                    return Err(self.error(format!("'{}' type is not legal",let_stem.ident).as_str()))
                }
                let_stem.token_type = self.current_tok.clone();
            }
            if !self.expect_peek_token(Token::Assign) {
                return Err(self.error("missing '=' symbol"))
            }
            if let_stem.token_type == Token::Unknown {
                if self.expect_peek_token(Token::Fun) {
                    return Ok(Statement::LetFun(let_stem.ident,self.fun_statement()?))
                }
                return Err(self.error("Unable to parse"))
            } else {
                self.next_token();
                let_stem.exp = Some(self.expression(Operation::Lowest)?);
            }
            self.dump_boundary();
            return Ok(Statement::Let(let_stem));
            // return Err(self.error(format!("let {} missing ':' ",&let_stem.ident).as_str()))
        }
        Err(self.error("let ? <- missing name!"))
    }
    fn list_value_expression(&mut self) -> Result<Expression,KzError>{
        self.expect_curr_token(Token::LeftSquareBra); //peek [
        let mut list:Vec<Box<Expression>> = Vec::new();

        while !self.current_token_is(Token::RightSquareBra) {
            list.push(Box::new(self.expression(Operation::Lowest)?));
            self.next_token();
            self.expect_curr_token(Token::Comma);
        }
        Ok(Expression::ListValue(list))
    }
    fn expression(&mut self,op:Operation) -> Result<Expression,KzError> {
        let mut left = match &self.current_tok {
            Token::Basics(val,tok) => self.basics(val,tok.as_ref())?,
            Token::Ident(val) => Expression::Ident(val.clone()),
            Token::LeftSquareBra => self.list_value_expression()?,
            _ => Expression::Unknown
        };
        if left == Expression::Unknown {
            return Err(self.error("expression Unknown"))
        }
        while op < self.peek_operation() && !self.peek_stem_end() {

             left = match &self.peek_tok {
                 Token::Add | Token::Sub |
                 Token::Div | Token::Mul |
                 Token::LT | Token::GT | Token::LT_Equ | Token::GT_Equ |
                 Token::BangEqu | Token::Equ
                 => {
                     self.next_token();
                     self.infix_expression(left)?
                 },
                 Token::LeftBracket => {
                     self.next_token();
                     let exp = self.call_expression(left)?;
                     self.expect_curr_token(Token::RightBracket);

                     exp
                 },
                 Token::LeftSquareBra => {
                     self.next_token();
                     self.index_expression(left)?
                 },
                 _ => {return Err(self.error(format!("unable to parse symbol '{:?}'",&self.peek_tok).as_str()))}
             };
        }
        Ok(left)
    }
    fn infix_expression(&mut self,left:Expression) -> Result<Expression,KzError> {
        let mut exp = InfixExpression::new(left);
        exp.op_symbol = self.current_tok.clone();
        self.next_token();
        exp.right = self.expression(self.curr_operation())?;

        Ok(Expression::Infix(Box::new(exp)))
    }
    fn basics(&self, val:&str, tok: &Token) -> Result<Expression,KzError>{
        if *tok == Token::Int {
            return  Ok(Expression::Int(val.parse::<i32>().unwrap()))
        }
        if *tok == Token::Float {
            return Ok(Expression::Float(val.parse::<f32>().unwrap()))
        }
        if *tok == Token::Bool {
            return Ok(Expression::Bool(val.parse::<bool>().unwrap()))
        }
        if *tok == Token::String {
            return Ok(Expression::String(val.to_string()))
        }
        return Err(self.error(format!("'{}' Is not a legal value",val).as_str()))
    }
    fn error(&self,err:&str) -> KzError {
        KzError::new(self.lex.file_path(),self.lex.line(),self.lex.line_index(),err)
    }

    fn peek_operation(&self) -> Operation {
        self.operation(&self.peek_tok)
    }
    fn curr_operation(&self) -> Operation {
        self.operation(&self.current_tok)
    }
    fn operation(&self, tok: &Token) -> Operation {
        match tok {
            Token::Add | Token::Sub => Operation::AddAndSub,
            Token::Div | Token::Mul => Operation::MulAndDiv,
            Token::LT | Token::GT  | Token::LT_Equ | Token::GT_Equ => Operation::LtAndGt,
            Token::Equ | Token::BangEqu => Operation::EquAls,
            Token::LeftBracket => Operation::Call,
            Token::LeftSquareBra => Operation::Index,
            _ => Operation::Lowest
        }
    }
    fn peek_stem_end(&self) -> bool {
        self.stem_end(&self.peek_tok)
    }
    fn curr_stem_end(&self) -> bool {
        self.stem_end(&self.current_tok)
    }
    fn stem_end(&self,tok: &Token) -> bool {
        match tok {
            Token::N | Token::Sem | Token::Eof => true,
            _ => false
        }
    }

    //if statement
    fn if_statement(&mut self) -> Result<Statement,KzError>{
        self.next_token();
        let exp = self.expression(Operation::Lowest)?;
        let mut if_stem = IfStatement::new(exp);
        if !self.expect_peek_token(Token::LeftCurlyBracket) {
            return Err(self.error("expected symbol '{' does not exist"))
        }
        self.next_token();

        if_stem.consequence = self.program()?;
        self.expect_peek_token(Token::RightCurlyBracket);
        if self.expect_peek_token(Token::Else) {
            self.next_token();
            while self.peek_token_is(Token::LeftCurlyBracket)
                || self.peek_token_is(Token::N) || self.current_token_is(Token::LeftCurlyBracket){
                self.next_token();
            }
            if_stem.alternative = self.program()?;
        }
        self.dump_token(Token::RightCurlyBracket);
        Ok(Statement::IF(if_stem))
    }
    //fn
    fn fun_head_statement(&mut self,need_param_name:bool) -> Result<FunStatement,KzError> {
        let name = match &self.peek_tok {
            Token::Ident(name) => {
                Some(name.clone())
            } ,
            _ => { None }
        };
        let mut fn_stem = FunStatement::new();
        self.next_token();
        if name != None {
            if !self.expect_peek_token(Token::LeftBracket) {
                return Err(self.error("expected symbol '(' does not exist"))
            }
            fn_stem.name = Some(Expression::Ident(name.unwrap()));
        }
        self.next_token();
        fn_stem.param_exp = Some(self.param_expression(Token::RightBracket,false,need_param_name)?);
        fn_stem.param_number = fn_stem.param_exp.as_ref().unwrap().len();
        self.expect_peek_token(Token::Arrow);
        self.next_token();
        if Token::let_type(&self.current_tok) {
            fn_stem.ret_type = Some(self.current_tok.clone());
        }
        Ok(fn_stem)
    }
    //fun statement
    fn fun_statement(&mut self) -> Result<FunStatement,KzError> {
        let mut fn_stem = self.fun_head_statement(true)?;
        if !self.expect_peek_token(Token::LeftCurlyBracket) {
            return Err(self.error("expected symbol '{' does not exist"))
        }
        self.next_token();
        fn_stem.body = self.program()?;

        self.dump_token(Token::RightCurlyBracket);
        Ok(fn_stem)
    }

    fn param_expression(&mut self, tok: Token,n:bool,need_name:bool) -> Result<Vec<Expression>,KzError>{
        let mut v = Vec::<Expression>::new();
        while !self.current_token_is(tok.clone()) {
            let name = match &self.current_tok {
                Token::Ident(val) => Some(val.clone()),
                _ => { None }
            };
            if self.current_tok == Token::N && n {
                self.next_token();
                continue
            }
            if name != None {
                if !self.expect_peek_token(Token::Colon)  {
                    return Err(self.error("expected symbol ':' does not exist"))
                }
                self.next_token();
                if self.current_token_is(Token::LeftSquareBra) {
                    self.array_token();
                }
                if !Token::let_type(&self.current_tok) {
                    return Err(self.error(format!("'{:?}' type is not legal",&self.current_tok).as_str()))
                }
                v.push(Expression::Param(name.unwrap(),self.current_tok.clone()));

                self.expect_peek_token(Token::Comma);
                self.next_token()
            } else {
                if !need_name {
                    //不需要名称的情况下
                    if self.current_token_is(Token::LeftSquareBra) {
                        self.array_token();
                    }
                    if !Token::let_type(&self.current_tok) {
                        return Err(self.error(format!("'{:?}' type is not legal",&self.current_tok).as_str()))
                    }
                    v.push(Expression::Param("".to_string(),self.current_tok.clone()));
                    self.expect_peek_token(Token::Comma);
                    self.next_token()
                } else {
                    return Err(self.error("Parameter name is not a valid value"))
                }
            }

        }
        Ok(v)
    }
    fn type_statement(&mut self) -> Result<Statement,KzError> {
        //type
        self.next_token();
        if let Token::Ident(name) = self.current_tok.clone(){
            if self.expect_peek_token(Token::Struct) {
                return Ok(Statement::Type(name.clone(),self.struct_expression()?));
            }
            if self.expect_peek_token(Token::Fun) {
                return Ok(Statement::Type(name.clone(),self.type_fn_expression()?));
            }
            if Token::let_type(&self.peek_tok) {
                let stem = Statement::Type(name.clone(),Expression::Basic(self.peek_tok.clone()));
                self.next_token();
                self.next_token();
                return Ok(stem)
            }
        }
        Err(self.error("type is missing name"))
    }
    fn struct_expression(&mut self) -> Result<Expression,KzError> {
        //struct 开始
        if !self.expect_peek_token(Token::LeftCurlyBracket) {
            return Err(self.error("expected symbol '{' does not exist"))
        }
        self.dump_n();//清除换行
        self.next_token();
        let param = self.param_expression(Token::RightCurlyBracket, true,true)?;

        self.dump_token(Token::RightCurlyBracket);
        Ok(Expression::Struct(Box::new(param)))
    }
    fn type_fn_expression(&mut self) -> Result<Expression,KzError> {
        //fun
        let f = self.fun_head_statement(false)?;
        if !self.current_token_is(Token::N) {
            self.next_token();
        }
        let mut p:Option<Box<Vec<Expression>>> = None;
        if f.param_exp != None {
            p = Some(Box::new(f.param_exp.unwrap()))
        }
        Ok(Expression::Fun(p,f.ret_type))
    }
    fn dump_n(&mut self) {
        while self.expect_peek_token(Token::N) {
        }
    }
    fn dump_token(&mut self,tok:Token) {
        loop {
            if self.current_token_is(tok.clone()) {
                self.next_token();
                break
            }
            self.next_token();
        }
    }
    fn dump_boundary(&mut self){
        loop {
            if self.current_token_is(Token::N) || self.current_token_is(Token::Sem)
                || self.current_token_is(Token::Eof) {
                 break
            }
            self.next_token();
        }
    }
    fn array_token(&mut self) {
       if self.current_token_is(Token::LeftSquareBra) {
           let number = match &self.peek_tok {
               Token::Basics(val,tok) => {
                   if *tok.as_ref() == Token::Int {
                       val.parse::<i32>().unwrap()
                   } else { 0 }
               }
               _ => {0}
           };
           if !self.peek_token_is(Token::RightSquareBra) {
               self.next_token();
           }
           if self.expect_peek_token(Token::RightSquareBra) {
               if Token::let_type(&self.peek_tok) {
                   self.next_token();
                   self.current_tok = Token::Array(number,Box::new(self.current_tok.clone()))
               }
           }
       }
    }
    fn ident_statement(&mut self) -> Result<Statement,KzError>{
        let exp = self.expression(Operation::Lowest)?;
        self.dump_boundary();
        Ok(Statement::Invoke(exp))
    }
    fn call_expression(&mut self,left:Expression) -> Result<Expression,KzError> {
        self.next_token();
        let mut list:Vec<Box<Expression>> = Vec::new();
        while !self.current_token_is(Token::RightBracket) && !self.curr_stem_end() {
            let exp = self.expression(Operation::Lowest)?;
            list.push(Box::new(exp));
            self.expect_peek_token(Token::Comma);
            self.next_token();
        }
        Ok(Expression::Call(Box::new(left),list))
    }

    fn index_expression(&mut self,left:Expression) -> Result<Expression,KzError> {
        self.next_token(); // [
        let exp = self.expression(Operation::Lowest)?;
        self.dump_token(Token::RightSquareBra);
        Ok(Expression::Index(Box::new(left),Box::new(exp)))
    }
}

#[test]
fn test_parser_02() {
    print_parser("./src/script/02_parser.kz");
}
#[test]
fn test_parser_03(){
    print_parser("./src/script/03_parser.kz");
}
#[test]
fn test_parser_04(){
    print_parser("./src/script/04_parser.kz");
}
#[test]
fn test_parser_05(){
    print_parser("./src/script/05_parser.kz");
}
#[test]
fn test_parser_06(){
    print_parser("./src/script/06_parser.kz");
}
#[test]
fn test_parser_07(){
    print_parser("./src/script/07_parser.kz");
}
#[test]
fn test_parser_08(){
    print_parser("./src/script/08_parser.kz");
}
fn print_parser(s:&str){
    let f = PaserFile::new(s);
    let  l =  Lexer::new(f);
    let mut p =  Parser::new(l);
    let stem = p.program();
    match stem {
        Ok(stem) => {
            for val in stem {
                println!("{:?}",val)
            }
        },
        Err(e) => { println!("{}",e.error())}
    }
}
