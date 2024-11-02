use derive_more::Display;
use std::rc::Rc;
use crate::function::Function;
use crate::ast::AST;
use std::fmt;

#[derive(Clone, PartialEq, Debug, Display)]
pub enum Token {
    // 字面量
    Float(f64),

    Identifier(String),
    Tuple,

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
    
    // 括号
    LParen, 
    RParen,

    
    // 分号
    Semicolon,

    // 语句
    Statement,

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
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Function(Rc<Function>),
    Tuple(Vec<Box<Value>>),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "{}",
            match self {
                Value::Number(n) => n.to_string(),
                Value::Boolean(boolean) => boolean.to_string(),
                Value::Tuple(tuple) => {
                   tuple.iter().map(|x| x.to_string()).collect()
                },
                Value::Function(_) => "Function".to_string(),
                Value::Null => "Null".to_string(),
            }
        )
    }
}

pub enum ControlFlow {
    Continue(Value),
    Return(Value),
}

impl ControlFlow {
    pub fn unwrap(self) -> Value {
        match self {
            ControlFlow::Continue(value) | ControlFlow::Return(value) => value,
        }
    }
}