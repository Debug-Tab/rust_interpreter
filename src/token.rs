use derive_more::Display;
use std::rc::Rc;
use crate::function::Function;

#[derive(Clone, PartialEq, Debug, Display)]
pub enum Token {
    // 字面量
    FLOAT(f64),

    IDENTIFIER(String),

    // 算数运算符
    PLUS,
    MINUS,

    MUL,
    DIV,

    MOD,

    // 逻辑运算符
    AND,
    OR,
    NOT,

    // 关系运算符
    EQUAL,
    UNEQUAL,
    GREATER,
    LESS,
    GREATER_EQUAL,
    LESS_EQUAL,


    // 赋值运算符
    ASSIGN,
    
    // 括号
    LPAREN, 
    RPAREN,

    
    // 分号
    SEMICOLON,

    // 语句
    STATEMENT,

    // 结束符
    EOF,

    // 函数
    FN,
    ARROW,     // 用于匿名函数 =>
    COMMA,
    LBRACE,
    RBRACE,
    RETURN,
    CALL,      // 用于标记函数调用
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Function(Rc<Function>),
    Null,
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