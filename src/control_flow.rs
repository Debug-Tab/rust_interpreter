use crate::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum ControlFlow {
    Value(Value),
    Continue,
    Return(Value),
    Break,
}

impl ControlFlow {
    pub fn value(self) -> Result<Value, String> {
        match self {
            ControlFlow::Value(value) => Ok(value),
            _ => Err(format!("Need expression, got {:?}!", self)),
        }
    }
}
