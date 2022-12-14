use crate::ast::for_::ForStatement;
use crate::ast::fun::{FunExpression};
use crate::ast::ifs::IfStatement;
use crate::ast::infix::InfixExpression;
use crate::ast::lets::LetStatement;
use crate::parser::file::PaserFile;
use crate::parser::lexer::Lexer;
use crate::parser::token::{Operation, Token};
use crate::ast::node::{Expression, Statement};
use crate::parser::error::{KzErr, KzError};



pub struct Parser {
    lex:Lexer,
    current_tok:Token,
    peek_tok:Token,
    notes:bool
}

impl Parser {
    pub fn new(l:Lexer) -> Parser {
       let mut p =  Self {
            lex:l,
            current_tok:Token::Err("parsing not started".to_string()),
            peek_tok:Token::Err("parsing not started".to_string()),
            notes:false,
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
        self.peek_tok = self.lex.next();
        self.notes()
    }
    fn notes(&mut self) {
        if self.notes {
            return;
        }
        self.notes = true;
        while self.current_token_is(Token::NotesBlock) || self.current_token_is(Token::Notes) {
            if self.current_token_is(Token::Eof) {
                break
            }
            match self.current_tok {
                Token::Notes => self.dump_notes_token(Token::N),
                Token::NotesBlock => self.dump_notes_token(Token::NotesBlockEnd),
                _ => {}
            }
        }
        self.notes = false;
    }
    pub fn program(&mut self) -> Result<Vec<Statement>,KzError> {
        let mut v = Vec::new();
        while !self.peek_token_is(Token::Eof) && !self.peek_token_is(Token::RightCurlyBracket) {
            match self.current_tok {
                Token::N => self.next_token(),
                Token::RightCurlyBracket => { return  Ok(v)},
                _ => {
                    v.push(self.parser_statement()?)
                }
            };
            if self.current_token_is(Token::Sem) {
                self.next_token();
            }
        }
        Ok(v)
    }
    fn parser_statement(&mut self) -> Result<Statement,KzError> {
        match self.current_tok {
            Token::Let => self.let_statement(),
            Token::If => self.if_statement(),
            Token::Fun => Ok(Statement::Fun(self.fun_expression()?)),
            Token::Type => self.type_statement(),
            Token::Ident(_) => self.ident_statement(),
            Token::SelfSub | Token::SelfAdd => self.prefix_statement(),
            Token::For => self.for_statement(),
            Token::Return | Token::Break | Token::Continue => self.rbc_statement(),
            _ => {
                return Err(self.error(KzErr::Program(self.current_tok.clone())))
            }
        }
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
                    return Err(self.error(KzErr::Type(self.current_tok.clone())))
                }
                let_stem.token_type = self.current_tok.clone();
            }
            if !self.expect_peek_token(Token::Assign) {
                return Err(self.error(KzErr::Expected(Token::Assign)))
            }
            self.next_token();
            let_stem.exp = Some(self.expression(Operation::Lowest)?);
            self.dump_boundary();
            return Ok(Statement::Let(let_stem));
        }
        Err(self.error(KzErr::ExpectedName))
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
            Token::Fun => {
               let exp = self.fun_expression()?;
               Expression::Fun(Box::new(exp))
            },
            _ => Expression::Unknown
        };
        if left == Expression::Unknown {
            return Err( self.error(KzErr::UnExpSymbol(self.current_tok.clone())))
        }
        while op < self.peek_operation() && !self.peek_stem_end() {

             left = match &self.peek_tok {
                 Token::Add | Token::Sub |
                 Token::Div | Token::Mul |
                 Token::LT | Token::GT | Token::LTEqu | Token::GTEqu |
                 Token::BangEqu | Token::Equ
                 => {
                     self.next_token();
                     self.infix_expression(left)?
                 },
                 Token::LeftBracket => {
                     self.next_token();
                     self.call_expression(left)?
                 },
                 Token::LeftSquareBra => {
                     self.next_token();
                     self.index_expression(left)?
                 },
                 Token::SelfAdd | Token::SelfSub => {
                     self.next_token();
                     self.self_operation_expression(left,false)?
                 },
                 _ => {return Err(self.error(KzErr::UnOpSymbol(self.peek_tok.clone())))}
             };
        }
        Ok(left)
    }
    fn infix_expression(&mut self,left:Expression) -> Result<Expression,KzError> {
        let mut exp = InfixExpression::new(left);
        exp.op_symbol = self.current_tok.clone();
        let op = self.curr_operation();
        self.next_token();
        exp.right = self.expression(op)?;
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

        Err(self.error(KzErr::Value(val.to_string())))
    }
    fn error(&self, err:KzErr) -> KzError {
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
            Token::Add | Token::Sub | Token::SelfSub | Token::SelfAdd => Operation::AddAndSub,
            Token::Div | Token::Mul => Operation::MulAndDiv,
            Token::LT | Token::GT  | Token::LTEqu | Token::GTEqu => Operation::LtAndGt,
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
            return Err(self.error(KzErr::Expected(Token::LeftCurlyBracket)))
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
    fn fun_head_expression(&mut self,need_param_name:bool) -> Result<FunExpression,KzError> {
        let name = match &self.peek_tok {
            Token::Ident(name) => {
                Some(name.clone())
            } ,
            _ => { None }
        };
        let mut fn_exp = FunExpression::new();
        self.next_token();
        if name != None {
            if !self.expect_peek_token(Token::LeftBracket) {
                return Err(self.error(KzErr::Expected(Token::LeftBracket)))
            }
            fn_exp.name = Some(Expression::Ident(name.unwrap()));
        }
        self.next_token();
        fn_exp.param_exp = Some(self.param_expression(Token::RightBracket,false,need_param_name)?);
        fn_exp.param_number = fn_exp.param_exp.as_ref().unwrap().len();
        self.expect_peek_token(Token::Arrow);
        self.next_token();
        if Token::let_type(&self.current_tok) {
            fn_exp.ret_type = Some(self.current_tok.clone());
        }
        Ok(fn_exp)
    }
    //fun expression
    fn fun_expression(&mut self) -> Result<FunExpression,KzError> {
        let mut fn_exp = self.fun_head_expression(true)?;
        if !self.expect_peek_token(Token::LeftCurlyBracket) {
            return Err(self.error(KzErr::Expected(Token::LeftCurlyBracket)))
        }
        self.next_token();
        fn_exp.body = self.program()?;

        self.dump_token(Token::RightCurlyBracket);
        Ok(fn_exp)
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
                    return Err( self.error(KzErr::Expected(Token::Colon)))
                }
                self.next_token();
                if self.current_token_is(Token::LeftSquareBra) {
                    self.array_token();
                }
                if !Token::let_type(&self.current_tok) {
                    return Err(self.error(KzErr::Type(self.current_tok.clone())))
                }
                v.push(Expression::Param(name.unwrap(),self.current_tok.clone()));

                self.expect_peek_token(Token::Comma);
                self.next_token()
            } else {
                if !need_name {
                    //???????????????????????????
                    if self.current_token_is(Token::LeftSquareBra) {
                        self.array_token();
                    }
                    if !Token::let_type(&self.current_tok) {
                        return Err( self.error(KzErr::Type(self.current_tok.clone())))
                    }
                    v.push(Expression::Param("".to_string(),self.current_tok.clone()));
                    self.expect_peek_token(Token::Comma);
                    self.next_token()
                } else {
                    return Err(self.error(KzErr::ParamName))
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
        Err( self.error(KzErr::ExpectedName))
    }
    fn struct_expression(&mut self) -> Result<Expression,KzError> {
        //struct ??????
        if !self.expect_peek_token(Token::LeftCurlyBracket) {
            return Err( self.error(KzErr::UnExpSymbol(Token::LeftCurlyBracket)))
        }
        self.dump_n();//????????????
        self.next_token();
        let param = self.param_expression(Token::RightCurlyBracket, true,true)?;

        self.dump_token(Token::RightCurlyBracket);
        Ok(Expression::Struct(Box::new(param)))
    }
    fn type_fn_expression(&mut self) -> Result<Expression,KzError> {
        //fun
        let f = self.fun_head_expression(false)?;
        if !self.current_token_is(Token::N) {
            self.next_token();
        }
        let mut p:Option<Box<Vec<Expression>>> = None;
        if f.param_exp != None {
            p = Some(Box::new(f.param_exp.unwrap()))
        }
        Ok(Expression::FunType(p,f.ret_type))
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
    fn dump_notes_token(&mut self,tok:Token) {
        loop {
            if self.current_token_is(tok.clone()) || self.current_token_is(Token::Eof) {
                self.next_token();
                break
            }
            self.next_token();
        }
    }
    fn dump_boundary(&mut self){
        loop {
            if self.current_token_is(Token::N) || self.current_token_is(Token::Sem)
                || self.current_token_is(Token::Eof) || self.current_token_is(Token::LeftCurlyBracket)
                || self.current_token_is(Token::RightCurlyBracket)
            {
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
        self.next_token();//(
        let mut list:Vec<Box<Expression>> = Vec::new();
        while !self.current_token_is(Token::RightBracket) && !self.curr_stem_end() {
            let exp = self.expression(Operation::Lowest)?;
            list.push(Box::new(exp));
            if self.expect_peek_token(Token::Comma) || self.expect_peek_token(Token::RightBracket) {
                self.next_token();
            }
        }
        self.expect_curr_token(Token::RightBracket);
        Ok(Expression::Call(Box::new(left),list))
    }

    fn index_expression(&mut self,left:Expression) -> Result<Expression,KzError> {
        self.next_token(); // [
        let exp = self.expression(Operation::Lowest)?;
        self.dump_token(Token::RightSquareBra);
        Ok(Expression::Index(Box::new(left),Box::new(exp)))
    }

    fn self_operation_expression(&mut self,name:Expression,left:bool) -> Result<Expression,KzError> {
        //-- ++
        let tok = self.current_tok.clone();
        Ok(Expression::SelfOp(Box::new(name),tok,left))
    }

    fn prefix_statement(&mut self) -> Result<Statement,KzError> {
        let tok = self.current_tok.clone();
        self.next_token();
        let exp = self.expression(Operation::Lowest)?;
        self.next_token();
        Ok(Statement::Invoke(Expression::SelfOp(Box::new(exp),tok,true)))
    }

    fn for_statement(&mut self) -> Result<Statement,KzError> {
        //for
        self.next_token();
        let start_condition = self.parser_statement()?;
        if !self.expect_curr_token(Token::Sem) {
            return Err(self.error(KzErr::UnExpSymbol(Token::Sem)))
        }
        let condition = self.expression(Operation::Lowest)?;
        self.next_token();
        if !self.expect_curr_token(Token::Sem) {
            return Err(self.error(KzErr::UnExpSymbol(Token::Sem)))
        }
        let self_operation = self.parser_statement()?;
        let mut for_stem = ForStatement::new(start_condition,condition,self_operation);
        if !self.expect_curr_token(Token::LeftCurlyBracket) {
            return Err(self.error(KzErr::UnExpSymbol(Token::LeftCurlyBracket)))
        }
        for_stem.consequence = self.program()?;
        self.dump_token(Token::RightCurlyBracket);
        Ok(Statement::For(for_stem))
    }

    //return break continue
    fn rbc_statement(&mut self) -> Result<Statement,KzError> {
        let tok = self.current_tok.clone();
        self.next_token();
        let stem = match tok {
            Token::Return => Statement::Return(self.expression(Operation::Lowest)?),
            Token::Break => Statement::Break,
            Token::Continue => Statement::Continue,
            _ => { return Err(self.error(KzErr::UnExpSymbol(tok)))}
        };
        self.dump_boundary();
        Ok(stem)
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
#[test]
fn test_parser_09(){
    print_parser("./src/script/09_parser.kz");
}
#[test]
fn test_parser_010(){
    print_parser("./src/script/10_parser.kz");
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
