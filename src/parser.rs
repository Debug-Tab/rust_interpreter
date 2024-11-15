use crate::lexer::Lexer;
use crate::token::Token;
use crate::ast_node::ASTNode;

use crate::debug;

pub struct Parser {
	pub lexer: Lexer,
	pub current_token: Option<Token>
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.get_next_token();
        Self { lexer, current_token: Some(current_token) }
    }

    fn cur_token_clone(&mut self) -> Option<Token> {
        self.current_token.clone()
    }

    fn cur_token_unwrap(&mut self) -> Token {
        self.cur_token_clone().unwrap_or_else(|| panic!("cur_token_unwrap: It's None!"))
    }

    fn eat(&mut self, expected_token: Token) -> Result<(), String> {
        if expected_token != self.cur_token_unwrap() {
            return Err(format!("Expected {}, found {:?}", expected_token, self.current_token));
        } 

        self.current_token = Some(self.lexer.get_next_token());
        Ok(())
    }

    fn next(&mut self) -> Option<Token> {
        self.current_token = Some(self.lexer.get_next_token());
        self.cur_token_clone()
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        debug!("{:?}", self.lexer.tokens.clone());
        self.statements()
    }

    fn statements(&mut self) -> Result<ASTNode, String> {
        let mut statements = vec![];
        
        while self.current_token != Some(Token::EOF) && self.current_token != Some(Token::RBrace) {
            let stmt = self.statement()?;
            statements.push(Box::new(stmt));
            
            if let Some(Token::Semicolon) = self.current_token {
                self.next();
            } else if self.current_token != Some(Token::EOF) && self.current_token != Some(Token::RBrace) {
                return Err(format!("Expected semicolon, found: {}!", self.cur_token_unwrap()));
            }
        }
        
        Ok(ASTNode::Block { statements })
    }

    fn statement(&mut self) -> Result<ASTNode, String> {
        debug!("{:?}", self.cur_token_clone());

        match self.current_token {
            Some(Token::FN) => self.function_definition(),

            Some(Token::If) => {
                self.next();
                let condition = Box::new(self.expression()?);
                let true_branch = Box::new(self.statement()?);
                let mut false_branch = None;
                if self.current_token == Some(Token::Else) {
                    self.next();
                    false_branch = Some(Box::new(self.statement()?));
                }
                Ok(ASTNode::Conditional { condition, true_branch, false_branch })
            },

            Some(Token::While) => {
                self.next();
                let condition = Box::new(self.expression()?);
                let body = Box::new(self.statement()?);
                Ok(ASTNode::Loop { condition, body })
            }

            Some(Token::Return) => {
                self.next();
                Ok(ASTNode::Return(self.statement()?.into()))
            },

            Some(Token::Let) => {
                self.next();
                let variables = self.identifier_list(false)?;

                Ok(ASTNode::Let { variables })
            },

            Some(Token::LBrace) => {
                self.next();
                let block = self.statements();
                self.eat(Token::RBrace)?;
                block
            },

            _ => {
                let mut node = self.expression()?;
                
                if self.current_token == Some(Token::Assign) {
                    self.eat(Token::Assign)?;
                    let value = Box::new(self.statement()?);

                    match node {
                        ASTNode::Identifier(name) => {
                            node = ASTNode::Assignment { name, value };
                        },
                        _ => {
                            return Err(format!("Invalid assignment to: {:?}!", node.clone()));
                        }
                    }
                    
                } else if self.current_token == Some(Token::Question) {
                    self.eat(Token::Question)?;
                    let left = self.statement()?;
                    self.eat(Token::Colon)?;
                    let right = self.statement()?;

                    node = ASTNode::Conditional { condition: Box::new(node), true_branch: Box::new(left), false_branch: Some(Box::new(right)) };
                }

                Ok(node)
            }
        }
    }

    fn expression(&mut self) -> Result<ASTNode, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<ASTNode, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<ASTNode, String> {
        let mut node = self.logical_and()?;

        while let Some(Token::Or) = self.current_token {
            self.next();
            let right = self.logical_and()?;
            node = ASTNode::LogicalOperation { operator: Token::Or, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn logical_and(&mut self) -> Result<ASTNode, String> {
        let mut node = self.equality()?;

        while let Some(Token::And) = self.current_token {
            self.next();
            let right = self.equality()?;
            node = ASTNode::LogicalOperation { operator: Token::And, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn equality(&mut self) -> Result<ASTNode, String> {
        let mut node = self.relational()?;

        while let token @ (Token::Equal | Token::UnEqual) = self.cur_token_unwrap() {
            self.next();
            let right = self.relational()?;
            node = ASTNode::LogicalOperation { operator: token, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn relational(&mut self) -> Result<ASTNode, String> {
        let mut node = self.additive_expression()?;

        while let token @ (Token::Greater | Token::Less | Token::GreaterEqual | Token::LessEqual) = self.cur_token_unwrap() {
            self.next();
            let right = self.additive_expression()?;
            node = ASTNode::LogicalOperation { operator: token, left: Box::new(node), right: Box::new(right) };
        }

        Ok(node)
    }

    fn additive_expression(&mut self) -> Result<ASTNode, String> {
        let mut node = self.multiplicative_expression()?;

        while let token @ (Token::Plus | Token::Minus) = self.cur_token_unwrap() {
            self.next();
            let right = self.multiplicative_expression()?;
            node = ASTNode::BinaryOperation { operator: token, left: node.into(), right: right.into() };
        }

        Ok(node)
    }

    fn multiplicative_expression(&mut self) -> Result<ASTNode, String> {
        let mut node = self.unary_expression()?;

        while let token @ (Token::Mul | Token::Div | Token::Mod) = self.cur_token_unwrap() {
            self.next();
            let right = self.unary_expression()?;
            node = ASTNode::BinaryOperation { operator: token, left: node.into(), right: right.into() };
        }

        Ok(node)
    }

    fn unary_expression(&mut self) -> Result<ASTNode, String> {
        if let token @ (Token::Plus | Token::Minus | Token::Not) = self.cur_token_unwrap() {
            self.next();
            let expr = self.unary_expression()?;
            Ok(ASTNode::UnaryOperation { operator: token, operand: Box::new(expr) })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<ASTNode, String> {
        let token = self.cur_token_unwrap();

        match token.clone() {
            Token::Identifier(name) => {
                self.next();

                if self.current_token == Some(Token::LParen) {
                    self.function_call(name)
                } else {
                    Ok(ASTNode::Identifier(name))
                }
            },

            Token::Float(_) | Token::String(_) | Token::True | Token::False | Token::Null => {
                self.next();
                Ok(ASTNode::Literal(token.to_value()?))
            },

            Token::LParen => {
                let tuple = self.collect_tuple(true)?;
                if tuple.len() == 1 {
                    Ok(*tuple[0].clone())
                } else {
                    Ok(ASTNode::Tuple(tuple))
                }
            },
            _ => Err(format!("[Parser] Unexpected token: {}!", self.cur_token_unwrap())),
        }
    }


    fn function_call(&mut self, name: String) -> Result<ASTNode, String> {
        self.eat(Token::LParen)?;
        let mut arguments = vec![];

        if self.current_token != Some(Token::RParen) {
            arguments.push(self.expression()?);
            while self.current_token == Some(Token::Comma) {
                self.next();
                arguments.push(self.expression()?);
            }
        }

        self.eat(Token::RParen)?;

        Ok(
            ASTNode::FunctionCall {
                function: Some(name),
                arguments,
            }
        )
    }

    fn function_definition(&mut self) -> Result<ASTNode, String> {
        self.eat(Token::FN)?;
        
        let name = if let Some(Token::Identifier(name)) = &self.current_token {
            let name = name.clone();
            self.next();
            Some(name)
        } else {
            None
        };

        let params = self.identifier_list(true)?;
        debug!("Params: {:?}", params);

        let body = self.statement()?;

        Ok(ASTNode::FunctionDefinition {
                name,
                params,
                body: Box::new(body),
            },
        )
    }

    fn identifier_list(&mut self, need_paren: bool) -> Result<Vec<String>, String> {
        let params = self.collect_tuple(need_paren)?;
        let mut result: Vec<String> = vec![];
        for i in params {
            match *i {
                ASTNode::Identifier(name) => result.push(name.clone()),
                _ => return Err(format!("Expected variable name, found: {:?}!", i))
            }
        }
        Ok(result)
    }

    fn collect_tuple(&mut self, need_paren: bool) -> Result<Vec<Box<ASTNode>>, String> {
        if need_paren { 
            self.eat(Token::LParen)?;
        }

        let mut tuple = vec![];

        if self.cur_token_unwrap() != Token::RParen {
            tuple.push(Box::new(self.statement().unwrap()));
            while self.cur_token_unwrap() == Token::Comma {
                self.next();
                tuple.push(Box::new(self.statement().unwrap()));
            }
        }

        if need_paren { 
            self.eat(Token::RParen)?;
        }
        Ok(tuple)
    }

    /*
    fn identifier(&mut self) -> Result<String, String> {
        if let Token::Identifier(name) = self.cur_token_unwrap() {
            self.next();
            Ok(name)
        } else {
            Err(format!("Expected identifier, found: {:?}!", self.cur_token_clone()))
        }
    }
     */
}
