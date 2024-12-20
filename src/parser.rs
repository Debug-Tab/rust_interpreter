use crate::lexer::Lexer;
use crate::token::Token;
use crate::ast_node::ASTNode;
use crate::debug;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Parser {
    tokens: Vec<Token>,
	pos: usize,
}

impl Parser {
    pub fn new(text: String) -> Result<Self, String> {
        Ok(Self { 
            tokens: Lexer::tokenize(text)?, 
            pos: 0,
        })
    }

    fn cur_token(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            Some(&self.tokens[self.pos])
        } else {
            None
        }
    }

    fn cur_token_clone(&mut self) -> Option<Token> {
        if let Some(token) = self.cur_token() {
            Some(token.clone())
        } else {
            None
        }
    }

    fn cur_token_equals(&mut self, token: Token) -> bool {
        self.cur_token() == Some(&token)
    }

    fn cur_token_in(&mut self, tokens: &[Token]) -> bool {
        let token = self.cur_token();
        if let Some(token) = token {
            tokens.contains(token)
        } else {
            false
        }
    }

    fn cur_token_is_not(&mut self, tokens: &[Token]) -> bool {
        let token = self.cur_token();
        if let Some(token) = token {
            !tokens.contains(token)
        } else {
            true
        }
    }

    fn cur_token_unwrap(&mut self) -> Token {
        self.cur_token_clone().expect("Current Token Unwrap(None)")
    }
    
    fn next(&mut self) {
        self.pos += 1;
    }

    fn next_token(&mut self) -> Option<Token> {
        self.next();
        self.cur_token_clone()
    }
    
    fn eat(&mut self, expected_token: Token) -> Result<(), String> {
        if expected_token != self.cur_token_unwrap() {
            return Err(format!("Expected {}, found {:?}", expected_token, self.cur_token()));
        } 

        self.next();
        Ok(())
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        debug!("{:?}", self.tokens);
        self.statements()
    }

    fn statements(&mut self) -> Result<ASTNode, String> {
        let mut statements = vec![];
        
        while self.cur_token_is_not(&[Token::EOF, Token::RBrace]) {
            let stmt = self.statement()?;
            statements.push(stmt);
            
            if self.cur_token_equals(Token::Semicolon) {
                self.next();
            } else if self.cur_token_is_not(&[Token::EOF, Token::RBrace]) {
                return Err(format!("Expected semicolon, found: {}!", self.cur_token_unwrap()));
            }
        }
        
        Ok(ASTNode::Block { statements: Box::new(statements) })
    }

    fn statement(&mut self) -> Result<ASTNode, String> {
        debug!("{:?}", self.cur_token_clone());

        if let Some(token) = self.cur_token() {
            match *token {
                Token::If => {
                    self.next();
                    let condition = Box::new(self.expression()?);
                    let true_branch = Box::new(self.statement()?);
                    let mut false_branch = None;
                    if self.cur_token_equals(Token::Else) {
                        self.next();
                        false_branch = Some(Box::new(self.statement()?));
                    }
                    Ok(ASTNode::Conditional { condition, true_branch, false_branch })
                },
    
                Token::While => {
                    self.next();
                    let condition = Box::new(self.expression()?);
                    let body = Box::new(self.statement()?);
                    Ok(ASTNode::Loop { condition, body })
                },
    
                Token::Break => {
                    self.next();
                    Ok(ASTNode::Break)
                }
    
                Token::Return => {
                    self.next();
                    Ok(ASTNode::Return(self.statement()?.into()))
                },
    
                Token::Let => {
                    self.next();
                    Ok(ASTNode::Let { ast: Box::new(self.expression()?) })
                    
                },
    
                Token::LBrace => {
                    self.next();
                    let block = self.statements();
                    self.eat(Token::RBrace)?;
                    block
                },
    
                _ => {
                    self.expression()
                }
            }
        } else {
            self.expression()
        }
        
    }

    fn expression(&mut self) -> Result<ASTNode, String> {
        match self.cur_token() {
            Some(Token::FN) => self.function_definition(),
            _ => {
                let mut node = self.assignment()?;

                if self.cur_token_equals(Token::Assign) {
                    self.next();
                    let value = Box::new(self.expression()?);

                    match node {
                        ASTNode::Identifier(name) => {
                            node = ASTNode::Assignment { name, value };
                        },
                        _ => {
                            return Err(format!("Invalid assignment to: {:?}!", node.clone()));
                        }
                    }
                } else if self.cur_token_equals(Token::Question) {
                    self.eat(Token::Question)?;
                    let left = self.expression()?;
                    self.eat(Token::Colon)?;
                    let right = self.expression()?;

                    node = ASTNode::Conditional { condition: Box::new(node), true_branch: Box::new(left), false_branch: Some(Box::new(right)) };
                }

                Ok(node)
            }
        }
    }

    fn assignment(&mut self) -> Result<ASTNode, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<ASTNode, String> {
        let mut node = self.logical_and()?;

        while self.cur_token_equals(Token::Or) {
            self.next();
            let right = self.logical_and()?;
            node = ASTNode::LogicalOperation { operator: Token::Or, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn logical_and(&mut self) -> Result<ASTNode, String> {
        let mut node = self.equality()?;

        while self.cur_token_equals(Token::And) {
            self.next();
            let right = self.equality()?;
            node = ASTNode::LogicalOperation { operator: Token::And, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn equality(&mut self) -> Result<ASTNode, String> {
        let mut node = self.relational()?;

        while let Some(token @ (Token::Equal | Token::UnEqual)) = self.cur_token_clone() {
            self.next();
            let right = self.relational()?;
            node = ASTNode::LogicalOperation { operator: token, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn relational(&mut self) -> Result<ASTNode, String> {
        let mut node = self.additive_expression()?;

        while let Some(token @ (Token::Greater | Token::Less | Token::GreaterEqual | Token::LessEqual)) = self.cur_token_clone() {
            self.next();
            let right = self.additive_expression()?;
            node = ASTNode::LogicalOperation { operator: token, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn additive_expression(&mut self) -> Result<ASTNode, String> {
        let mut node = self.multiplicative_expression()?;

        while let Some(token @ (Token::Plus | Token::Minus)) = self.cur_token_clone() {
            self.next();
            let right = self.multiplicative_expression()?;
            node = ASTNode::BinaryOperation { operator: token, left: node.into(), right: right.into() };
        }

        Ok(node)
    }

    fn multiplicative_expression(&mut self) -> Result<ASTNode, String> {
        let mut node = self.unary_expression()?;

        while let Some(token @ (Token::Mul | Token::Div | Token::Mod)) = self.cur_token_clone() {
            self.next();
            let right = self.unary_expression()?;
            node = ASTNode::BinaryOperation { operator: token, left: node.into(), right: right.into() };
        }

        Ok(node)
    }

    fn unary_expression(&mut self) -> Result<ASTNode, String> {
        if let Some(token @ (Token::Plus | Token::Minus | Token::Not)) = self.cur_token_clone() {
            self.next();
            let expr = self.index_expression()?;
            Ok(ASTNode::UnaryOperation { operator: token, operand: Box::new(expr) })
        } else {
            self.index_expression()
        }
    }

    fn index_expression(&mut self) -> Result<ASTNode, String> {
        let mut result = self.primary()?;

        while self.cur_token_equals(Token::LBracket) {
            self.next();
            result = ASTNode::Index { expression: Box::new(result), index: Box::new(self.expression()?) };
            self.eat(Token::RBracket)?;
        }

        Ok(result)
    }

    fn primary(&mut self) -> Result<ASTNode, String> {
        let token = self.cur_token_unwrap();

        match token.clone() {
            Token::Identifier(name) => {
                self.next();

                if self.cur_token_equals(Token::LParen) {
                    self.function_call(*name)
                } else {
                    Ok(ASTNode::Identifier(name))
                }
            },

            Token::Float(_) | Token::String(_) | Token::True | Token::False | Token::Null => {
                self.next();
                Ok(ASTNode::Literal(token.to_value()?))
            },

            Token::LParen => {
                let tuple = self.tuple()?;
                if tuple.len() == 1 {
                    Ok(tuple[0].clone())
                } else {
                    Ok(ASTNode::Tuple(tuple))
                }
            },

            Token::LBracket => {
                Ok(ASTNode::Vector(self.vector()?))
            },

            _ => Err(format!("[Parser] Unexpected token: {}!", self.cur_token_unwrap())),
        }
    }


    fn function_call(&mut self, name: String) -> Result<ASTNode, String> {
        Ok(
            ASTNode::FunctionCall {
                function: Some(name),
                arguments: self.tuple()?,
            }
        )
    }

    fn function_definition(&mut self) -> Result<ASTNode, String> {
        self.eat(Token::FN)?;

        let params = self.identifier_list()?;
        debug!("Params: {:?}", params);

        let body = self.statement()?;

        Ok(ASTNode::FunctionDefinition {
                params,
                body: Box::new(body),
            },
        )
    }

    fn identifier_list(&mut self) -> Result<Vec<String>, String> {
        let params = self.tuple()?;
        let result: Vec<String> = 
            params
                .iter()
                .map(|ast| {
                    match ast {
                        ASTNode::Identifier(name) => *name.clone(),
                        _ => panic!("Expected variable name, found: {:?}!", ast),
                    }
                })
                .collect()
                ;
        
        Ok(result)
    }

    fn tuple(&mut self) -> Result<Box<Vec<ASTNode>>, String> {
        self.eat(Token::LParen)?;
        let result = self.collect_list();
        self.eat(Token::RParen)?;
        result
    }

    fn vector(&mut self) -> Result<Box<Vec<ASTNode>>, String> {
        self.eat(Token::LBracket)?;
        let result = self.collect_list();
        self.eat(Token::RBracket)?;
        result
    }

    fn set(&mut self) -> Result<Box<Vec<ASTNode>>, String> {
        self.eat(Token::LBrace)?;
        let result = self.collect_list();
        self.eat(Token::RBrace)?;
        result
    }

    fn collect_list(&mut self) -> Result<Box<Vec<ASTNode>>, String> {
        let mut list = vec![];

        list.push(self.expression()?);
        while self.cur_token_unwrap() == Token::Comma {
            self.next();
            list.push(self.expression()?);
        }
        
        Ok(Box::new(list))
    }
}
