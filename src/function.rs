use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::ast_node::ASTNode;
use crate::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Function {
    UserDefined {
        params: Vec<String>,
        body: Box<ASTNode>,
        closure: Rc<RefCell<Environment>>,
    },
    BuiltIn {
        func: fn(Vec<Value>) -> Result<Value, String>,
    },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            values: HashMap::new(),
            parent: None,
        };

        env.define("printf".to_string(), 
            Value::Function(
                Rc::new(
                    Function::BuiltIn { func: |args| {
                            if let Value::String(format) = &args[0] {
                                let formatted = format_string(format, &args[1..])?;
                                print!("{}", formatted);
                                Ok(Value::Null)
                            } else {
                                Err(format!("The first argument must be a string, actually found: {}", args[0]))
                            }
                        }
                    }
                )
            )
        );

        env
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


fn format_string(format: &str, args: &[Value]) -> Result<String, String> {
    let mut result = String::new();
    let mut arg_index = 0;

    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'}') {
                chars.next();
                if arg_index < args.len() {
                    result.push_str(&args[arg_index].to_string());
                    arg_index += 1;
                } else {
                    return Err("Not enough arguments for format string".to_string());
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    if arg_index < args.len() {
        Err(format!("Too many arguments for format string. Need {}, found {}", arg_index, args.len()))
    } else {
        Ok(result)
    }
}
