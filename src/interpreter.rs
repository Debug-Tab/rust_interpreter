use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::debug;
use crate::pre_include::initialization;
use crate::parser::Parser;
use crate::token::Token;
use crate::value::Value;
use crate::control_flow::ControlFlow;
use crate::pre_include::hole_func;
use crate::ast_node::{ASTNode, AstRef};
use crate::environment::Environment;

#[derive(Serialize, Deserialize, Debug)]
pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Box::new(Environment::new()),
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        initialization(&mut self.environment)?;
        Ok(())
    }


    pub fn interpret(&mut self, text: String) -> Result<Value, String> {
        let ast = Parser::new(text)?.parse()?;
        debug!("ast: {:?}", ast);
        Ok(self.evaluate(&ast)?.unwrap())
    }

    pub fn evaluate(&mut self, node: &ASTNode) -> Result<ControlFlow, String> {
        let result = match node {
            ASTNode::Block { statements } => {
                let mut result = ControlFlow::Continue(Value::Null);

                for statement in statements {
                    result = self.evaluate(statement)?;
                    if let ControlFlow::Return(_) = result {
                        break;
                    }
                }

                result
            },


            ASTNode::Let { ast } => {
                match *ast.clone() {
                    ASTNode::Identifier(name) => self.environment.define(name, Value::Null)?,
                    ASTNode::Assignment { name, value } => {
                        let value = self.evaluate_expression(&value)?;
                        self.environment.define(name, value)?
                    },
                    _ => return Err(format!("Cannot binding this: {:?}", ast)),
                }
                
                ControlFlow::Continue(Value::Null)
            },

            ASTNode::Conditional { condition, true_branch, false_branch } => {
                let condition_value = self.evaluate(condition)?;
                if let Value::Boolean(true) = condition_value.unwrap() {
                    self.evaluate(true_branch)?
                } else {
                    if let Some(false_branch) = false_branch {
                        self.evaluate(false_branch)?
                    } else {
                        ControlFlow::Continue(Value::Null)
                    }
                }
            },

            ASTNode::Loop { condition, body } => {
                let mut result = ControlFlow::Continue(Value::Null);

                while let Value::Boolean(true) = self.evaluate_expression(condition)? {
                    result = self.evaluate(body)?;
                    match result {
                        ControlFlow::Return(_) => return Ok(result),
                        ControlFlow::Continue(_) => (),
                        ControlFlow::Break => break,
                    }
                }

                result
            },

            ASTNode::Break => {
                ControlFlow::Break
            }

            ASTNode::Return(expr) => {
                let value = self.evaluate_expression(expr)?;
                ControlFlow::Return(value)
            },

            _ => {
                ControlFlow::Continue(self.evaluate_expression(node)?)
            },
        };
        
        Ok(result)
    }

    
    fn evaluate_expression(&mut self, node: &ASTNode) -> Result<Value, String> {
        let result = match node {
            ASTNode::BinaryOperation { operator, left, right } => {
                let left = self.evaluate_expression(&left)?;
                let right = self.evaluate_expression(&right)?;

                let result = match (left.clone(), right.clone()) {
                    (Value::Number(left), Value::Number(right)) => {
                        match operator {
                            Token::Plus => left + right,
                            Token::Minus => left - right,
                            Token::Mul => left * right,
                            Token::Div => if right == 0.0 { return Err("Division by zero!".to_string()) } else { left / right },
                            Token::Mod => if right == 0.0 { return Err("Modulo by zero".to_string()) } else { left % right },

                            _ => {
                                return Err(format!("Invalid operator for binary operation: {:?}", operator))
                            },
                        }
                    },
                    _ => {
                        return Err(format!("Invalid operands for binary operation: {:?} {:?}", left, right))
                    }
                };
                
                Value::Number(result)
            },

            ASTNode::LogicalOperation { operator, left, right } => {
                let result = match operator {
                    Token::And => {
                        if self.evaluate_expression(&left)?.get_boolean()? {
                            if self.evaluate_expression(&right)?.get_boolean()? {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },

                    Token::Or => {
                        if self.evaluate_expression(&left)?.get_boolean()? {
                            true
                        } else if self.evaluate_expression(&right)?.get_boolean()? {
                            true
                        } else {
                            false
                        }
                    },

                    _ => {
                        let left: Value = self.evaluate_expression(&left)?.into();
                        let right: Value = self.evaluate_expression(&right)?.into();

                        match operator {
                            Token::Equal => left.equal(right)?,
                            Token::UnEqual => !left.equal(right)?,

                            _ => {
                                let left: f64 = left.to_number()?;
                                let right: f64 = right.to_number()?;

                                match operator {
                                    Token::Greater => {
                                        left > right
                                    }, 
                                    Token::Less => {
                                        left < right
                                    }, 
                                    Token::GreaterEqual => {
                                        left >= right
                                    }, 
                                    Token::LessEqual => {
                                        left <= right
                                    }, 
                                    _ => {
                                        return Err(format!("Invalid operator for binary operation: {:?}", operator))
                                    }
                                }
                                
                            }
                        }
                    },
                };

                Value::Boolean(result)
            },

            ASTNode::UnaryOperation { operator, operand } => {
                let operand_value = self.evaluate_expression(operand)?;

                match operator {
                    Token::Plus => {
                        operand_value
                    },
                    Token::Minus => {
                        if let Value::Number(v) = operand_value {
                            Value::Number(-v)
                        } else {
                            return Err(format!("Invalid operand for unary minus: {:?}", operand_value))
                        }
                    },
                    Token::Not => {
                        if let Value::Boolean(v) = operand_value {
                            Value::Boolean(!v)
                        } else {
                            return Err(format!("Invalid operand for logical NOT: {:?}", operand_value))
                        }
                    },
                    _ => {
                        return Err(format!("Invalid operator for unary operation: {:?}", operator))
                    }
                }
            },

            ASTNode::Literal(value) => {
                value.clone()
            },

            ASTNode::Identifier(name) => {
                self.get_variable_value(name)?
            },

            ASTNode::Tuple(tuple) => {
                let mut result: Vec<Value> = vec![];

                for i in tuple {
                    result.push(self.evaluate_expression(i)?.into());
                }

                Value::Tuple(result)
            },

            ASTNode::Assignment { name, value } => {
                let evaluated_value = self.evaluate_expression(value)?;
                self.environment.set(name.clone(), evaluated_value.clone())?;
                evaluated_value
            },

            ASTNode::FunctionDefinition { params, body } => {
                Value::Function{
                    params: params.clone(),
                    body: Box::clone(body),
                    closure: self.environment.clone(),
                }
            },

            ASTNode::FunctionCall { function, arguments } => {
                if let Some(function) = function {
                    let result = self.evaluate_function_call(self.get_variable_value(function)?, arguments)?;
                    result
                } else {
                    return Err(format!("Expected String, found: null"))
                }
                
            },

            _ => return Err(format!("{:?} is not an expression", node))
        };

        Ok(result)
    }
    

    fn evaluate_function_call<T: AstRef>(&mut self, function: Value, arguments: &[T]) -> Result<Value, String> {

        if let Value::Function { params, body, closure } = function.clone() {
            if params.len() != arguments.len() {
                return Err(format!("Function expected {} arguments, but got {}", params.len(), arguments.len()));
            }
            
            let mut new_env = Environment {
                values: HashMap::new(),
                parent: Some(closure.clone()),
            };
            
            for (param, arg) in params.iter().zip(arguments) {
                let arg_value = self.evaluate_expression(arg.as_ast())?;
                new_env.values.insert(param.clone(), arg_value);
            }
            new_env.define("self".to_string(), function.clone())?;
            
            let old_env = std::mem::replace(&mut self.environment, Box::new(new_env));
            let result = self.evaluate(&body);
            self.environment = old_env;
            
            match result {
                Ok(c) => Ok(c.unwrap()),
                Err(e) => Err(e),
            } 
        } else if let Value::Hole(id) = function {
            let args: Vec<Value> = arguments.iter()
                .map(|arg| self.evaluate_expression(arg.as_ast()))
                .collect::<Result<Vec<Value>, String>>()?;
            return hole_func(id, args);
        } else {
            Err("Attempted to call a non-function value".to_string())
        }
    }


    fn get_variable_value(&self, name: &str) -> Result<Value, String> {
        self.environment.get(name)
    }
}
