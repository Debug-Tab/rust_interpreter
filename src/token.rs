use derive_more::Display;
use std::rc::Rc;
use crate::function::Function;
use crate::ast::AST;
use std::fmt;

#[derive(Clone, PartialEq, Debug, Display)]
pub enum Token {
    // 字面量
    FLOAT(f64),

    IDENTIFIER(String),
    TUPLE,

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
    ARROW,
    COMMA,
    LBRACE,
    RBRACE,
    RETURN,
    CALL,
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