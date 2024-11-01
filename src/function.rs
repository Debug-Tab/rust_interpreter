use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::ast::AST;
use crate::token::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Box<AST>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}