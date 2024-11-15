use crate::value::Value;

#[derive(Clone, PartialEq, Debug)]
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

impl From<ControlFlow> for Value {
    fn from(value: ControlFlow) -> Self {
        match value {
            ControlFlow::Continue(v) => v,
            ControlFlow::Return(v) => v,
        }
    }
}

impl From<ControlFlow> for bool {
    fn from(c: ControlFlow) -> Self {
        let v: Value = c.into();
        v.into()
    }
}