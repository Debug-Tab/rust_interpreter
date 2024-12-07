use crate::token::Token;
use crate::value::Value;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ASTNode {
    FunctionDefinition {
        params: Vec<String>,
        body: Box<ASTNode>,
    },

    FunctionCall {
        function: Option<String>,
        arguments: Vec<ASTNode>,
    },

    BinaryOperation {
        operator: Token,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },

    LogicalOperation {
        operator: Token,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },

    UnaryOperation {
        operator: Token,
        operand: Box<ASTNode>,
    },

    Block {
        statements: Vec<Box<ASTNode>>,
    },

    Assignment {
        name: String,
        value: Box<ASTNode>,
    },

    Let {
        ast: Box<ASTNode>,
    },

    Conditional {
        condition: Box<ASTNode>,
        true_branch: Box<ASTNode>,
        false_branch: Option<Box<ASTNode>>,
    },

    Loop {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },

    Literal(Value),
    Identifier(String),
    Tuple(Vec<Box<ASTNode>>),
    Return(Box<ASTNode>),
}

pub trait AstRef {
    fn as_ast(&self) -> &ASTNode;
}

impl AstRef for ASTNode {
    fn as_ast(&self) -> &ASTNode {
        self
    }
}

impl AstRef for Box<ASTNode> {
    fn as_ast(&self) -> &ASTNode {
        self.as_ref()
    }
}

