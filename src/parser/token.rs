use std::fmt::{Debug, Formatter};

#[derive(Clone,PartialEq)]
pub enum Token {
    Eof,
    Let,
    Equ,
    Ident(String),
    Colon,
    If,
    Else,
    Unknown,
    Basics(String,Box<Token>),
    Sem,
    N,
    Err(String),
    Int,
    String,
    Float,
    Bool,
    Add,
    Sub,
    Mul,
    Div,
    LeftCurlyBracket,
    RightCurlyBracket,
    LT,
    GT,
    LTEqu,
    GTEqu,
    BangEqu,
    Bang,
    Assign,
    Fun,
    LeftBracket,
    RightBracket,
    Comma,
    Arrow,
    Type,
    Struct,
    LeftSquareBra,
    RightSquareBra,
    Array(i32,Box<Token>),
    For,
    SelfSub,
    SelfAdd,
    Break,
    Continue,
    Return,
    Notes,
    NotesBlock,
    NotesBlockEnd,
}
impl Token {
    pub fn let_type(tok:&Token) -> bool {
         match tok {
             Token::Int | Token::String | Token::Float  => true,
             Token::Ident(_) => true,
             Token::Array(_,_) => true,
             _ => false
         }
    }
}
//if else for true false
impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String;
         write!(f,"{}",match self {
             Token::Let => "let",
             Token::Eof => "eof",
             Token::Equ => "==",
             Token::Ident(ident) => {
                 s = format!("Ident<{}>",ident);
                 s.as_str()
             },
             Token::Colon => ":",
             Token::If => "if",
             Token::Else => "else",
             Token::Unknown => "unknown",
             Token::Sem => "sem",
             Token::N => "<N>",
             Token::Err(val) => {
                 s = format!("Error => {}",val);
                 s.as_str()
             },
             Token::Basics(val,tok) => {
                 s = format!("Basics<{}> type {:?}",val,tok);

                 s.as_str()
             },
             Token::String => "string",
             Token::Int => "int",
             Token::Float => "float",
             Token::Bool => "bool",
             Token::Div => "/",
             Token::Add => "+",
             Token::Sub => "-",
             Token::Mul => "*",
             Token::LeftCurlyBracket => "{",
             Token::RightCurlyBracket => "}",
             Token::LT => "<",
             Token::GT => ">",
             Token::LTEqu => "<=",
             Token::GTEqu => ">=",
             Token::BangEqu => "!=",
             Token::Bang => "!",
             Token::Assign => "=",
             Token::Fun => "fun",
             Token::LeftBracket => "(",
             Token::RightBracket => ")",
             Token::Comma => ",",
             Token::Arrow => "->",
             Token::Struct => "struct",
             Token::Type => "type",
             Token::LeftSquareBra => "[",
             Token::RightSquareBra => "]",
             Token::Array(number,tok) => {
                 s = format!("[{}]{:?}",number,tok);
                 s.as_str()
             },
             Token::For => "for",
             Token::SelfSub => "--",
             Token::SelfAdd => "++",
             Token::Break => "break",
             Token::Continue => "continue",
             Token::Return => "return",
             Token::Notes => "//",
             Token::NotesBlock => "/**",
             Token::NotesBlockEnd => "**/"
         })
    }
}
#[derive(PartialOrd,PartialEq)]
pub enum Operation {
    Lowest,
    Index,
    EquAls,
    AddAndSub,
    MulAndDiv,
    LtAndGt,
    Call
}

