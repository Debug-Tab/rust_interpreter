use crate::value::Value;

pub enum ControlFlow {
    Continue(Value),
    Return(Value),
}

impl ControlFlow {
    pub fn unwrap(self) -> Value {
        match self {
            ControlFlow::Continue(value) | ControlFlow::Return(value) => value,
        }
    }
}

impl From<ControlFlow> for bool {
    fn from(value: ControlFlow) -> Self {
        match value {
            ControlFlow::Continue(v) => v.into(),
            ControlFlow::Return(_v) => false,
        }
    }
}