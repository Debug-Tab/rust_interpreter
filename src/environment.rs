use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::value::Value;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        let env = Self {
            values: HashMap::new(),
            parent: None,
        };
        /*
        env.define("printf".to_string(), 
            Value::Function(
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
        );*/

        env
    }

    pub fn get(&self, name: &str) -> Result<Value, String> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else if let Some(parent) = self.parent.as_mut() {
            parent.set(name, value)
        } else {
            Err(format!("Variable {} not declared.", name))
        }
    }

    pub fn define(&mut self, name: String, value: Value) -> Result<(), String>{
        if !self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else {
            Err(format!("Variable {} have been declared!", name))
        }
        
    }
}

