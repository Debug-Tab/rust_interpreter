use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::debug;

use crate::parser::Parser;

use crate::token::Token;
use crate::value::Value;
use crate::control_flow::ControlFlow;

use crate::ast::{AST, ASTNode, AstRef};
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

    fn evaluate(&mut self, node: &AST) -> Result<ControlFlow, String> {
        match &node.node {
            Some(ASTNode::FunctionDefinition { name, params, body }) => {
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
            Some(ASTNode::FunctionCall { function, arguments }) => {
                let result = self.evaluate_function_call(function, arguments)?;
                Ok(ControlFlow::Continue(result))
            },
            Some(ASTNode::Return(expr)) => {
                let value = self.evaluate_expression(expr)?;
                Ok(ControlFlow::Return(value))
            },
            _ => {
                let result = self.evaluate_expression(node)?;
                Ok(ControlFlow::Continue(result))
            }
        }
    }

    fn evaluate_expression(&mut self, node: &AST) -> Result<Value, String> {
        match &node.token {
            Token::Call => {
                let function = &node.children[0];
                let arguments = &node.children[1].children;
                self.evaluate_function_call(function, arguments.as_slice())
            },
            Token::Statement => {
                let mut result = ControlFlow::Continue(Value::Number(0.0));
                for child in &node.children {
                    result = self.evaluate(child)?;
                }
                match result {
                    ControlFlow::Continue(value) | ControlFlow::Return(value) => Ok(value),
                }
            },
            Token::Assign => self.evaluate_assignment(node),
            Token::Let => self.declare_variables(node),
            Token::Identifier(name) => self.get_variable_value(name),

            Token::Float(value) => Ok(Value::Number(*value)),
            Token::Tuple => {
                let mut tuple = vec![];
                for child in &node.children {
                    tuple.push(self.evaluate_expression(child).unwrap());
                }
                Ok(Value::Tuple(tuple))
            },
            Token::True => Ok(Value::Boolean(true)),
            Token::False => Ok(Value::Boolean(false)),
            
            Token::Plus => {
                if node.children.len() == 1 {
                    self.evaluate_unary_op(node, |v| v)
                } else {
                    self.evaluate_binary_op(node, |a, b| Ok(a + b))
                }
            },
            Token::Minus => {
                if node.children.len() == 1 {
                    self.evaluate_unary_op(node, |v| -v)
                } else {
                    self.evaluate_binary_op(node, |a, b| Ok(a - b))
                }
            },

            Token::Mul => self.evaluate_binary_op(node, |a, b| Ok(a * b)),
            Token::Div => self.evaluate_binary_op(node, |a, b| {
                if b == 0.0 { Err("Division by zero!".to_string()) } else { Ok(a / b) }
            }),

            Token::Mod => self.evaluate_binary_op(node, |a, b| {
                if b == 0.0 { Err("Modulo by zero".to_string()) } else { Ok(a % b) }
            }),

            Token::Not => self.evaluate_unary_op(node, |v| if v == 0.0 { 1.0 } else { 0.0 }),

            Token::And => self.evaluate_logical_op(node),
            Token::Or => self.evaluate_logical_op(node),

            Token::Equal => self.evaluate_comparison_op(node, |a, b| (a - b).abs() < f64::EPSILON),
            Token::UnEqual => self.evaluate_comparison_op(node, |a, b| (a - b).abs() >= f64::EPSILON),

            Token::Greater => self.evaluate_comparison_op(node, |a, b| a > b),
            Token::Less => self.evaluate_comparison_op(node, |a, b| a < b),

            Token::GreaterEqual => self.evaluate_comparison_op(node, |a, b| a >= b),
            Token::LessEqual => self.evaluate_comparison_op(node, |a, b| a <= b),

            _ => Err(format!("Unexpected token: {}!", node.token)),
        }
    }

    fn evaluate_function_call<T: AstRef>(&mut self, function: &AST, arguments: &[T]) -> Result<Value, String> {
        let function_value = self.evaluate_expression(function)?;

        if let Value::Function(func) = function_value {
            if func.params.len() != arguments.len() {
                return Err(format!("Function expected {} arguments, but got {}", func.params.len(), arguments.len()));
            }
            
            let mut new_env = Environment {
                values: HashMap::new(),
                parent: Some(func.closure.clone()),
            };
            
            for (param, arg) in func.params.iter().zip(arguments) {
                let arg_value = self.evaluate_expression(arg.as_ast())?;
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

    fn evaluate_binary_op<F>(&mut self, node: &AST, op: F) -> Result<Value, String>
    where
        F: FnOnce(f64, f64) -> Result<f64, String>,
    {
        let left = self.evaluate(&node.children[0])?;
        let right = self.evaluate(&node.children[1])?;
        match (left.unwrap(), right.unwrap()) {
            (Value::Number(left), Value::Number(right)) => op(left, right).map(Value::Number),
            _ => Err("Invalid operands for binary operation".to_string()),
        }
    }

    fn evaluate_unary_op<F>(&mut self, node: &AST, op: F) -> Result<Value, String>
    where
        F: FnOnce(f64) -> f64,
    {
        let value = self.evaluate(&node.children[0])?;
        match value.unwrap() {
            Value::Number(v) => Ok(Value::Number(op(v))),
            Value::Boolean(b) => Ok(Value::Boolean(if op(if b {1.0} else {0.0})!=0.0 {true} else {false})),
            _ => Err("Invalid operand for unary operation".to_string()),
        }
    }

    fn evaluate_logical_op(&mut self, node: &AST) -> Result<Value, String> {
        match &node.token {
            Token::And => {
                if self.evaluate(&node.children[0])?.into() {
                    if self.evaluate(&node.children[1])?.into() {
                        return Ok(Value::Boolean(true))
                    }
                }
                return Ok(Value::Boolean(false))
            },
            Token::Or => {
                if self.evaluate(&node.children[0])?.into() {
                    return Ok(Value::Boolean(true))
                }
                if self.evaluate(&node.children[1])?.into() {
                    return Ok(Value::Boolean(true))
                } 
                return Ok(Value::Boolean(false))
            },
            _ => Err(format!("[evaluate_logical_op] The AST that should not call this function: {}", &node))
        }
    }

    fn evaluate_comparison_op<F>(&mut self, node: &AST, op: F) -> Result<Value, String>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let left = self.evaluate(&node.children[0])?;
        let right = self.evaluate(&node.children[1])?;
        match (left.unwrap(), right.unwrap()) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(op(left, right))),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(op(if left {1.0} else {0.0}, if right {1.0} else {0.0}))),
            _ => Err("Invalid operands for logical operation".to_string()),
        }
    }

    fn declare_variables(&mut self, node: &AST) -> Result<Value, String> {
        for variable in &node.children {
            if let Token::Identifier(name) = &variable.token {
                self.environment.borrow_mut().values.insert(name.clone(), Value::Null);
            } else {
                return Err(format!("Invalid assignment to: {}.", variable.token))
            }
        }
        Ok(Value::Null)
        
    }

    fn evaluate_assignment(&mut self, node: &AST) -> Result<Value, String> {
        if let Token::Identifier(name) = &node.children[0].token {
            let value = self.evaluate(&node.children[1])?.unwrap();
            self.environment.borrow_mut().set(name.clone(), value.clone())?;
            Ok(value)
        } else {
            Err(format!("Invalid assignment to: {}.", node.children[0].token))
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
