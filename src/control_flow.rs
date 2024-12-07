use crate::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum ControlFlow {
    Continue(Value),
    Return(Value),
    Break,
}

impl ControlFlow {
    pub fn unwrap(self) -> Value {
        match self {
            ControlFlow::Continue(value) | ControlFlow::Return(value) => value,
            ControlFlow::Break => Value::Null,
        }
    }
}

impl From<ControlFlow> for Value {
    fn from(value: ControlFlow) -> Self {
        match value {
            ControlFlow::Continue(v) => v,
            ControlFlow::Return(v) => v,
            ControlFlow::Break => Value::Null,
        }
    }
}