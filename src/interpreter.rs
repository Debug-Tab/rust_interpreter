use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::debug;
use crate::builtin::initialization;
use crate::parser::Parser;
use crate::token::Token;
use crate::value::Value;
use crate::control_flow::ControlFlow;
use crate::builtin::hole_func;
use crate::ast_node::ASTNode;
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


    pub fn interpret(&mut self, text: String) -> Result<ControlFlow, String> {
        let ast = Parser::parse(text)?;
        debug!("ast: {:?}", ast);
        Ok(self.evaluate(&ast)?)
    }

    pub fn evaluate(&mut self, node: &ASTNode) -> Result<ControlFlow, String> {
        let result = match node {
            ASTNode::Block { statements, will_return } => {
                let mut result = ControlFlow::Continue;

                for statement in *statements.clone() {
                    result = self.evaluate(&statement)?;
                    if let ControlFlow::Return(_) = result {
                        return Ok(result);
                    } else if result == ControlFlow::Break {
                        break;
                    }
                }

                if *will_return {
                    result
                } else {
                    ControlFlow::Continue
                }
                
            },


            ASTNode::Let { ast } => {
                match *ast.clone() {
                    ASTNode::Identifier(name) => self.environment.define(*name, Value::Null)?,
                    ASTNode::Assignment { name, value } => {
                        let value = self.evaluate_expression(&value)?;
                        self.environment.define(*name, value)?
                    },
                    _ => return Err(format!("Cannot binding this: {:?}", ast)),
                }
                
                ControlFlow::Continue
            },

            ASTNode::Conditional { condition, true_branch, false_branch } => {
                let condition_value = self.evaluate(condition)?;

                if let Value::Boolean(true) = condition_value.value()? {
                    self.evaluate(true_branch)?
                } else {
                    if let Some(false_branch) = false_branch {
                        self.evaluate(false_branch)?
                    } else {
                        ControlFlow::Continue
                    }
                }
            },

            ASTNode::Loop { condition, body } => {
                let mut result;

                while let Value::Boolean(true) = self.evaluate_expression(condition)? {
                    result = self.evaluate(body)?;
                    match result {
                        ControlFlow::Return(_) => return Ok(result),
                        ControlFlow::Break => break,
                        ControlFlow::Continue | ControlFlow::Value(_)=> (),
                    }
                }

                ControlFlow::Continue
            },

            ASTNode::Break => {
                ControlFlow::Break
            }

            ASTNode::Return(expr) => {
                let value = self.evaluate_expression(expr)?;
                return Ok(ControlFlow::Return(value))
            },

            _ => {
                ControlFlow::Value(self.evaluate_expression(node)?)
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

                for i in *tuple.clone() {
                    result.push(self.evaluate_expression(&i)?);
                }

                Value::Tuple(result)
            },

            ASTNode::Vector(vector) => {
                let mut result: Vec<Value> = vec![];

                for i in *vector.clone() {
                    result.push(self.evaluate_expression(&i)?);
                }

                Value::Vector(result)
            },

            ASTNode::Index { expression, index } => {
                let expression = self.evaluate_expression(expression)?;
                let index = self.evaluate_expression(index)?;

                match expression {
                    Value::Tuple(list) | Value::Vector(list) => {
                        match index {
                            Value::Number(num) => {
                                let index = num as usize;
                                if index < list.len() {
                                    list[index as usize].clone()
                                } else {
                                    return Err(format!("Index out of bounds: the len is {} but the index is {}", list.len(), index));
                                }
                                
                            }, // todo: not true
                            _ => return Err(format!("This expression cannot be used as an index: {index}")),
                        }
                    },
                    _ => return Err(format!("This expression cannot be indexed: {expression}")),
                }
            },

            ASTNode::Assignment { name, value } => {
                let evaluated_value = self.evaluate_expression(value)?;
                self.environment.set(*name.clone(), evaluated_value.clone())?;
                evaluated_value
            },

            ASTNode::Function { params, body } => {
                Value::Function{
                    params: params.clone(),
                    body: Box::clone(body),
                    closure: self.environment.clone(),
                }
            },

            ASTNode::FunctionCall { function, arguments } => {
                let func = self.evaluate_expression(&*function.clone())?;
                self.evaluate_function_call(func, arguments)?
            },

            _ => return Err(format!("{:?} is not an expression", node))
        };

        Ok(result)
    }
    

    fn evaluate_function_call(&mut self, function: Value, arguments: &[ASTNode]) -> Result<Value, String> {

        if let Value::Function { params, body, closure } = function.clone() {
            if params.len() != arguments.len() {
                return Err(format!("Function expected {} arguments, but got {}", params.len(), arguments.len()));
            }
            
            let mut new_env = Environment {
                values: HashMap::new(),
                parent: Some(closure.clone()),
            };
            
            for (param, arg) in params.iter().zip(arguments) {
                let arg_value = self.evaluate_expression(arg)?;
                new_env.values.insert(param.clone(), arg_value);
            }
            new_env.define("self".to_string(), function.clone())?;
            
            let old_env = std::mem::replace(&mut self.environment, Box::new(new_env));
            
            let result = match self.evaluate(&body)? {
                ControlFlow::Return(v) | ControlFlow::Value(v) => v,
                _ => Value::Null,
            };

            self.environment = old_env;
            Ok(result)

        } else if let Value::Hole(id) = function {
            let args: Vec<Value> = arguments.iter()
                .map(|arg| self.evaluate_expression(arg))
                .collect::<Result<Vec<Value>, String>>()?;
            hole_func(id, args)

        } else {
            Err("Attempted to call a non-function value".to_string())
        }
    }


    fn get_variable_value(&self, name: &str) -> Result<Value, String> {
        self.environment.get(name)
    }
}
