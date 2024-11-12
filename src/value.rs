use std::rc::Rc;
use crate::function::Function;
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Function(Rc<Function>),
    Tuple(Vec<Value>),
    Null,
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(i) => i != 0.0,
            Value::Boolean(b) => b,
            Value::Function(_) => true,
            Value::Tuple(v) => v.len() != 0,
            Value::Null => false,
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
                Value::Tuple(tuple) => {
                    format!("({})", tuple.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))
                },
                Value::Function(_) => "Function".to_string(),
                Value::Null => "Null".to_string(),
            }
        )
    }
}