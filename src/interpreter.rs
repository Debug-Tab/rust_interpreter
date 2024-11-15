use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::debug;

use crate::parser::Parser;

use crate::token::Token;
use crate::value::Value;
use crate::control_flow::ControlFlow;

use crate::ast_node::{ASTNode, AstRef};
use crate::function::{Function, Environment};

pub struct Interpreter {
    pub parser: Parser,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Self {
            parser,
            environment: Rc::new(RefCell::new(Environment {
                values: HashMap::new(),
                parent: None,
            })),
        }
    }

    pub fn interpret(&mut self) -> Result<Value, String> {
        let ast = self.parser.parse()?;
        debug!("ast: {:?}", ast);
        match self.evaluate(&ast)? {
            ControlFlow::Continue(value) | ControlFlow::Return(value) => Ok(value),
        }
    }

    fn evaluate(&mut self, ast: &ASTNode) -> Result<ControlFlow, String> {
        match ast {
            ASTNode::Block { statements } => {
                let mut result = ControlFlow::Continue(Value::Null);

                for statement in statements {
                    result = self.evaluate(statement)?;
                    if let ControlFlow::Return(_) = result {
                        break;
                    }
                }

                Ok(result)
            },

            ASTNode::FunctionDefinition { name, params, body } => {
                let function = Rc::new(Function {
                    params: params.clone(),
                    body: Box::clone(body),
                    closure: self.environment.clone(),
                });
                let value = Value::Function(function);
                
                if let Some(name) = name {
                    self.environment.borrow_mut().define(name.clone(), value.clone());
                }
                
                Ok(ControlFlow::Continue(value))
            },

            ASTNode::FunctionCall { function, arguments } => {
                if let Some(function) = function {
                    let result = self.evaluate_function_call(self.get_variable_value(function)?, arguments)?;
                    Ok(ControlFlow::Continue(result))
                } else {
                    Err(format!("Expected String, found: null"))
                }
                
            },

            ASTNode::BinaryOperation { operator, left, right } => {
                let left = self.evaluate(&left)?.unwrap();
                let right = self.evaluate(&right)?.unwrap();

                let result = match (left.clone(), right.clone()) {
                    (Value::Number(left), Value::Number(right)) => {
                        match operator {
                            Token::Plus => left + right,
                            Token::Minus => left - right,
                            Token::Mul => left * right,
                            Token::Div => if right == 0.0 { return Err("Division by zero!".to_string()) } else { left / right },
                            Token::Mod => if right == 0.0 { return Err("Modulo by zero".to_string()) } else { left % right },

                            _ => {return Err(format!("Invalid operator for binary operation: {:?}", operator))},
                        }
                    },
                    _ => { return Err(format!("Invalid operands for binary operation: {:?} {:?}", left, right)) }
                };
                
                Ok(ControlFlow::Continue(Value::Number(result)))
            },

            ASTNode::LogicalOperation { operator, left, right } => {
                let result = match operator {
                    Token::And => {
                        if self.evaluate(&left)?.into() {
                            if self.evaluate(&right)?.into() {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },

                    Token::Or => {
                        if self.evaluate(&left)?.into() {
                            true
                        } else if self.evaluate(&right)?.into() {
                            true
                        } else {
                            false
                        }
                    },

                    _ => {
                        let left: Value = self.evaluate(&left)?.into();
                        let right: Value = self.evaluate(&right)?.into();

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
                                    _ => { return Err(format!("Invalid operands for binary operation: {:?} {:?}", left, right)) }
                                }
                                
                            }
                        }
                    },
                };

                Ok(ControlFlow::Continue(Value::Boolean(result)))
            },

            ASTNode::UnaryOperation { operator, operand } => {
                let operand_value = self.evaluate(operand)?;

                match operator {
                    Token::Plus => {
                        Ok(operand_value)
                    },
                    Token::Minus => {
                        if let ControlFlow::Continue(Value::Number(v)) = operand_value {
                            Ok(ControlFlow::Continue(Value::Number(-v)))
                        } else {
                            Err(format!("Invalid operand for unary minus: {:?}", operand_value))
                        }
                    },
                    Token::Not => {
                        if let ControlFlow::Continue(Value::Boolean(v)) = operand_value {
                            Ok(ControlFlow::Continue(Value::Boolean(!v)))
                        } else {
                            Err(format!("Invalid operand for logical NOT: {:?}", operand_value))
                        }
                    },
                    _ => {
                        Err(format!("Invalid operator for unary operation: {:?}", operator))
                    }
                }
            },

            ASTNode::Literal(value) => {
                Ok(ControlFlow::Continue(value.clone()))
            },

            ASTNode::Identifier(name) => {
                Ok(ControlFlow::Continue(self.get_variable_value(name)?))
            },

            ASTNode::Tuple(tuple) => {
                let mut result: Vec<Value> = vec![];

                for i in tuple {
                    result.push(self.evaluate(i)?.into());
                }

                Ok(ControlFlow::Continue(Value::Tuple(result)))
            }

            ASTNode::Assignment { name, value } => {
                let evaluated_value = self.evaluate(value)?;
                self.environment.borrow_mut().set(name.clone(), evaluated_value.clone().unwrap())?;
                Ok(evaluated_value)
            },

            ASTNode::Let { variables } => {
                for var in variables {
                    self.environment.borrow_mut().define(var.clone(), Value::Null);
                }
                Ok(ControlFlow::Continue(Value::Null))
            },

            ASTNode::Conditional { condition, true_branch, false_branch } => {
                let condition_value = self.evaluate(condition)?;
                if let Value::Boolean(true) = condition_value.unwrap() {
                    self.evaluate(true_branch)
                } else {
                    if let Some(false_branch) = false_branch {
                        self.evaluate(false_branch)
                    } else {
                        Ok(ControlFlow::Continue(Value::Null))
                    }
                }
            },

            ASTNode::Loop { condition, body } => {
                while let Value::Boolean(true) = self.evaluate(condition)?.unwrap() {
                    self.evaluate(body)?;
                }
                Ok(ControlFlow::Continue(Value::Null))
            },

            ASTNode::Return(expr) => {
                let value = self.evaluate(expr)?.unwrap();
                Ok(ControlFlow::Return(value))
            },
        }
        
        
    }

    /*
    fn evaluate_expression(&mut self, node: &ASTNode) -> Result<Value, String> {
        Ok(Value::Null)
    }
    */

    fn evaluate_function_call<T: AstRef>(&mut self, function: Value, arguments: &[T]) -> Result<Value, String> {
        let function_value = function;

        if let Value::Function(func) = function_value {
            if func.params.len() != arguments.len() {
                return Err(format!("Function expected {} arguments, but got {}", func.params.len(), arguments.len()));
            }
            
            let mut new_env = Environment {
                values: HashMap::new(),
                parent: Some(func.closure.clone()),
            };
            
            for (param, arg) in func.params.iter().zip(arguments) {
                let arg_value = self.evaluate(arg.as_ast())?.unwrap();
                new_env.values.insert(param.clone(), arg_value);
            }
            
            let old_env = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(new_env)));
            let result = self.evaluate(&func.body);
            self.environment = old_env;
            
            match result {
                Ok(ControlFlow::Return(value)) | Ok(ControlFlow::Continue(value)) => Ok(value),
                Err(e) => Err(e),
            }
        } else {
            Err("Attempted to call a non-function value".to_string())
        }
    }


    fn get_variable_value(&self, name: &str) -> Result<Value, String> {
        let mut current_env = self.environment.clone();
        loop {
            if let Some(value) = current_env.borrow().values.get(name) {
                return Ok(value.clone());
            }

            let parent = current_env.borrow().parent.clone();
            match parent {
                Some(i) => current_env = i,
                None => return Err(format!("Undefined variable: {}", name)),
            }
        }
    }
}
