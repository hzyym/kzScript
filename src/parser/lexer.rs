
use super::{file::PaserFile, token::Token};
pub struct Lexer {
    f:PaserFile,
    index:usize,
    ch:u8,
    line:i32,
    line_index:i32,
}
impl Lexer {
    pub fn new(file:PaserFile) -> Lexer {
        Self {
            f:file,
            index:0,
            ch:0,
            line:1,
            line_index:0,
        }
    }

    pub fn next(&mut self) -> Token {
        if self.index >= self.f.body.len() {
            return Token::Eof;
        }
        loop {
            self.read();
            self.line_index += 1;
            if self.ch == b' ' {
                continue;
            }
            if self.ch == b'\r' {
                self.line +=1;
                self.line_index = 0;
                continue;
            }
            break;
        };

        if self.is_letter(self.ch) {
            let ident = self.read_string();
            return self.as_token(ident.as_str());
        }
        if self.is_num(self.ch) {
            return self.read_num();
        }
        match self.ch {
            b':' => Token::Colon,
            b'\n' => Token::N,
            b';'=>Token::Sem,
            b'=' => {
                if self.expect_peek(b'=') {
                    return Token::Equ
                }
                Token::Assign
            },
            b'+' => {
                if self.expect_peek(b'+') {
                    return Token::SelfAdd
                }
                Token::Add
            },
            b'-' => {
                if self.expect_peek(b'>') {
                    return Token::Arrow
                }
                if self.expect_peek(b'-') {
                    return Token::SelfSub
                }
                Token::Sub
            },
            b'*' => {
                if self.expect_peek(b'*') {
                    if self.expect_peek(b'/') {
                        return Token::NotesBlockEnd
                    }
                    return Token::Err("notes block end not '*/' the is a '**/'".to_string())
                }
                Token::Mul
            },
            b'/' => {
                if self.expect_peek(b'/') {
                    return Token::Notes
                }
                if self.expect_peek(b'*') {
                    if self.expect_peek(b'*') {
                        return Token::NotesBlock
                    }
                    return Token::Err("notes block not '/*' the is a '/**'".to_string())
                }
                Token::Div
            },
            b'{' => Token::LeftCurlyBracket,
            b'}' => Token::RightCurlyBracket,
            b'<' => {
                if self.expect_peek(b'='){
                    return Token::LTEqu
                }
                Token::LT
            },
            b'>' => {
                if self.expect_peek(b'='){
                    return Token::GTEqu
                }
                Token::GT
            },
            b'!' => {
                if self.expect_peek(b'=') {
                    return Token::BangEqu
                }
                Token::Bang
            },
            b'(' => Token::LeftBracket,
            b')' => Token::RightBracket,
            b',' => Token::Comma,
            b'"' => {
                if self.peek() == b'"' {
                    return Token::Basics("".to_string(),Box::new(Token::String))
                }
                self.read();
                let s = self.read_strings();
                if self.expect_peek(b'"') {
                    return Token::Basics(s.to_string(),Box::new(Token::String))
                }

                Token::Err("is not the expected string type".to_string())
            }
            b'[' => Token::LeftSquareBra,
            b']' => Token::RightSquareBra,
            _ => Token::Unknown
        }
    }
    
    fn read(&mut self) {
        if  self.index >= self.f.body.len() {
            return
        }
        self.ch = self.f.body[self.index];
        self.index += 1;
    }

    fn peek(&self) -> u8 {
        if self.index >= self.f.body.len() {
            return 0;
        }
        self.f.body[self.index]
    }
    fn expect_peek(&mut self,symbol:u8) -> bool{
        if self.peek() == symbol {
            self.read();
            //self.read();
            return true
        }
        false
    }
    fn is_letter(&mut self,val:u8) -> bool {
        val >= b'a' && val <= b'z' || val >= b'A' && val <= b'Z' || val == b'_'
    }

    fn is_num(&self,val:u8) -> bool {
        val >= b'0' && val <= b'9'
    }
    fn read_string(&mut self) -> String {
       
        let mut char:Vec<u8> = Vec::new();
        char.push(self.ch);
        
        while self.is_letter(self.peek()) {
            self.read();
            char.push(self.ch);
        }

        return String::from_utf8( char).unwrap();
    }
    fn read_strings(&mut self) -> String{

        let mut char:Vec<u8> = Vec::new();
        char.push(self.ch);

        while self.peek() != b'"' {
            self.read();
            char.push(self.ch);
        }

        return String::from_utf8( char).unwrap();
    }
    fn read_num(&mut self) -> Token {
        let mut char:Vec<u8> = vec![self.ch];
        let mut tok = Token::Int;
        while self.is_num(self.peek()) || self.peek() == b'.'{
            self.read();
            char.push(self.ch);
            if self.ch == b'.' { tok = Token::Float }
        }
        if self.is_letter(self.ch) {
            return  Token::Err("the number cannot contain other char".to_string());
        }
        Token::Basics(String::from_utf8(char).unwrap(),Box::new(tok))
    }
    fn as_token(&self,ident:&str) -> Token {
         match ident {
            "let" => Token::Let,
             "if" => Token::If,
             "else" => Token::Else,
             "true" => Token::Basics("true".to_string(), Box::new(Token::Bool)),
             "false" => Token::Basics("false".to_string(),Box::new(Token::Bool)),
             "string"=> Token::String,
             "int" => Token::Int,
             "float"=>Token::Float,
             "fun" => Token::Fun,
             "type"=> Token::Type,
             "struct"=>Token::Struct,
             "for"=> Token::For,
             "continue"=> Token::Continue,
             "break" => Token::Break,
             "return"=> Token::Return,
             _ => Token::Ident(ident.to_string()),
        }
    }

    pub fn line(&self) ->i32 {
        return self.line
    }
    pub fn line_index(&self) -> i32 {
        self.line_index
    }
    pub fn file_path(&self) -> &str {
        self.f.file_path()
    }
}

#[test]
fn test_lexer() {
    let f = PaserFile::new("./src/script/01.kz");
    let mut l = Lexer::new(f);
    loop {
        let tok =  l.next();
        match tok {
            Token::Eof => break,
            _ => println!("{:?}",tok)
        }
    }

}
