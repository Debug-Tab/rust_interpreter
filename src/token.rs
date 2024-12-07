use derive_more::Display;
use serde::{Serialize, Deserialize};

use crate::value::Value;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Display)]
pub enum Token {
    // 字面量
    Float(f64),
    Tuple,
    String(String),

    Identifier(String),

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

    
    // 分号
    Semicolon,

    // 语句
    Statement,
    Break,

    // 结束符
    EOF,

    // 函数
    FN,
    Arrow,
    Comma,
    LBrace,
    RBrace,
    Return,
    Call,

    Question,
    Colon,
    
    If,
    Else,
    While,
}

impl Token {
    pub fn to_value(&self) -> Result<Value, String> {
        Ok(match self {
            Token::Float(v) => Value::Number(*v),
            Token::String(str) => Value::String(str.clone()),
            Token::True => Value::Boolean(true),
            Token::False => Value::Boolean(false),
            Token::Null => Value::Null,
            _ => return Err(format!("Could not convert this to Value: {:?}", self.clone()))
        })
    }
}