use crate::function::Function;

use serde::{Serialize, Deserialize};
use std::fmt::{self};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Function),
    Hole(u32),
    Tuple(Vec<Value>),
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
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(i) => i != 0.0,
            Value::Boolean(b) => b,
            Value::String(str) => str.len() != 0,
            Value::Function(_) => true,
            Value::Hole(_) => true,
            Value::Tuple(v) => v.len() != 0,
            Value::Null => false,
            Value::Nothing => false,
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
                Value::String(str) => str.clone(),
                Value::Tuple(tuple) => {
                    format!("({})", tuple.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))
                },
                Value::Function(_) => "Function".to_string(),
                Value::Hole(v) => format!("<Builtin Function (Hole{})>", v),
                Value::Null => "Null".to_string(),
                Value::Nothing => String::new(),
            }
        )
    }
}