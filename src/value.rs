use crate::ast_node::ASTNode;
use crate::environment::Environment;

use serde::{Serialize, Deserialize};
use std::fmt::{self};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(Box<String>),
    Function {
        params: Vec<String>,
        body: Box<ASTNode>,
        closure: Box<Environment>,
    },
    Hole(u32),
    Tuple(Vec<Value>),
    Vector(Vec<Value>),
    Null,
    Nothing,
}

impl Value {
    pub fn equal(&self, other: Self) -> Result<bool, String> {
        match (self.clone(), other.clone()) {
            (Value::Number(a), Value::Number(b)) => Ok((a - b).abs() < f64::EPSILON),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(a == b),
            _ => Err(format!("Could not compare: {:?} {:?}", *self, other))
        }
    }

    pub fn to_number(&self) -> Result<f64, String> {
        match *self {
            Value::Number(n) => Ok(n),
            Value::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            _ => Err(format!("Could not convert to number: {:?}", self.clone())),
        }
    }

    pub fn get_boolean(&self) -> Result<bool, String> {
        match *self {
            Value::Boolean(b) => Ok(b),
            _ => Err(format!("Expected bool, found: {}!", self.clone())),
        }
    }
}


impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "{}",
            match self {
                Value::Number(n) => n.to_string(),
                Value::Boolean(boolean) => boolean.to_string(),
                Value::String(str) => *str.clone(),
                Value::Tuple(tuple) => {
                    format!("({})", tuple.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))
                },
                Value::Vector(vector) => {
                    format!("[{}]", vector.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))
                },
                Value::Function { .. } => "Function".to_string(),
                Value::Hole(v) => format!("<Builtin Function (Hole{})>", v),
                Value::Null => "Null".to_string(),
                Value::Nothing => String::new(),
            }
        )
    }
}