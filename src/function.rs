use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::ast::AST;
use crate::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Box<AST>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn get(&self, name: &str) -> Result<Value, String> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            Err(format!("Variable {} not declared", name))
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}
