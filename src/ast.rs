use crate::Token;
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub struct AST {
    pub token: Token,
    pub children: Vec<Box<AST>>,
    pub node: Option<ASTNode>,  // 新增字段
}

impl AST {
    pub fn new(token: Token, children: Vec<Box<AST>>) -> Self {
        Self { token, children, node: None }
    }

    pub fn with_node(token: Token, children: Vec<Box<AST>>, node: ASTNode) -> Self {
        Self { token, children, node: Some(node) }
    }
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.children.len() {
            0 => write!(f, "AST({})", self.token),
            _ => {
                let children_str: Vec<String> = self.children.iter().map(|c| c.to_string()).collect();
                write!(f, "AST({}, [{}])", self.token, children_str.join(", "))
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ASTNode {
    FunctionDefinition {
        name: Option<String>,
        params: Vec<String>,
        body: Box<AST>,
    },
    FunctionCall {
        function: Box<AST>,
        arguments: Vec<AST>,
    },
    Return(Box<AST>),
}

pub trait AstRef {
    fn as_ast(&self) -> &AST;
}

impl AstRef for AST {
    fn as_ast(&self) -> &AST {
        self
    }
}

impl AstRef for Box<AST> {
    fn as_ast(&self) -> &AST {
        self.as_ref()
    }
}

