use derive_more::Display;
use serde::{Serialize, Deserialize};


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Display)]
pub enum Token {
    // 字面量
    Float(f64),
    Tuple,
    String(Box<String>),

    Identifier(Box<String>),

    True,
    False,
    Null,

    // 算数运算符
    Plus,
    Minus,

    Mul,
    Div,

    Mod,

    // 逻辑运算符
    And,
    Or,
    Not,

    // 关系运算符
    Equal,
    UnEqual,
    
    Greater,
    Less,
    GreaterEqual,
    LessEqual,


    // 赋值运算符
    Assign,
    Let,
    
    // 括号
    LParen, 
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    
    // 分号
    Semicolon,

    // 语句
    Statement,
    Break,

    // 结束符
    EOF,

    // 函数
    FN,
    Lambda,
    Arrow,
    Comma,
    Return,
    Call,

    Question,
    Colon,
    
    If,
    Else,
    While,
}
