use derive_more::Display;

#[derive(Clone, PartialEq, Debug, Display)]
pub enum Token {
    // 字面量
    Float(f64),

    Identifier(String),
    Tuple,

    True,
    False,

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
