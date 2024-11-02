use crate::lexer::Lexer;
use crate::token::Token;
use crate::ast::AST;
use crate::ast::ASTNode;
use std::rc::Rc;

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

    pub fn parse(&mut self) -> Result<AST, String> {
        self.statements()
    }

    fn statements(&mut self) -> Result<AST, String> {
        let mut statements = vec![];
        
        while self.current_token != Some(Token::EOF) && self.current_token != Some(Token::RBrace) {
            let stmt = self.statement()?;
            statements.push(Box::new(stmt));
            
            if let Some(Token::Semicolon) = self.current_token {
                self.next();
            } else if self.current_token != Some(Token::EOF) && self.current_token != Some(Token::RBrace) {
                return Err("Expected semicolon.".to_string());
            }
        }
        
        Ok(AST::new(Token::Statement, statements))
    }

    fn statement(&mut self) -> Result<AST, String> {
        match self.current_token {
            Some(Token::FN) => self.function_definition(),
            Some(Token::LBrace) => {
                self.next();
                let block = self.statements();
                self.eat(Token::RBrace)?;
                block
            }
            _ => {
                let mut node = self.expression()?;

                if let Some(Token::Assign) = self.current_token {
                    self.eat(Token::Assign)?;
                    let expr = self.statement()?;
                    node = AST::new(Token::Assign, vec![Box::new(node), Box::new(expr)]);
                }

                Ok(node)
            }
        }
    }

    fn expression(&mut self) -> Result<AST, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<AST, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<AST, String> {
        let mut node = self.logical_and()?;

        while let Some(Token::Or) = self.current_token {
            self.next();
            let right = self.logical_and()?;
            node = AST::new(Token::Or, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn logical_and(&mut self) -> Result<AST, String> {
        let mut node = self.equality()?;

        while let Some(Token::And) = self.current_token {
            self.next();
            let right = self.equality()?;
            node = AST::new(Token::And, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn equality(&mut self) -> Result<AST, String> {
        let mut node = self.relational()?;

        while let token @ (Token::Equal | Token::UnEqual) = self.cur_token_unwrap() {
            self.next();
            let right = self.relational()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn relational(&mut self) -> Result<AST, String> {
        let mut node = self.additive_expression()?;

        while let token @ (Token::Greater | Token::Less | Token::GreaterEqual | Token::LessEqual) = self.cur_token_unwrap() {
            self.next();
            let right = self.additive_expression()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn additive_expression(&mut self) -> Result<AST, String> {
        let mut node = self.multiplicative_expression()?;

        while let token @ (Token::Plus | Token::Minus) = self.cur_token_unwrap() {
            self.next();
            let right = self.multiplicative_expression()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn multiplicative_expression(&mut self) -> Result<AST, String> {
        let mut node = self.unary_expression()?;

        while let token @ (Token::Mul | Token::Div | Token::Mod) = self.cur_token_unwrap() {
            self.next();
            let right = self.unary_expression()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn unary_expression(&mut self) -> Result<AST, String> {
        if let token @ (Token::Plus | Token::Minus | Token::Not) = self.cur_token_unwrap() {
            self.next();
            let expr = self.unary_expression()?;
            Ok(AST::new(token, vec![Box::new(expr)]))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<AST, String> {
        match &self.current_token {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.next();
                if self.current_token == Some(Token::LParen) {
                    // 函数调用
                    self.function_call(name)
                } else {
                    Ok(AST::new(Token::Identifier(name), vec![]))
                }
            },
            Some(Token::Float(value)) => {
                let value = *value;
                self.next();
                Ok(AST::new(Token::Float(value), vec![]))
            },
            Some(Token::LParen) => {
                let tuple = self.collect_tuple()?;
                if tuple.len() == 1 {
                    Ok(*tuple[0].clone())
                } else {
                    Ok(AST::new(Token::Tuple, tuple))
                }
            },
            _ => Err(format!("Unexpected token: {}!", self.cur_token_unwrap())),
        }
    }

    fn function_call(&mut self, name: String) -> Result<AST, String> {
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

        Ok(AST::with_node(
            Token::Call,
            vec![Box::new(AST::new(Token::Identifier(name.clone()), vec![])), 
            Box::new(AST::new(Token::Statement, arguments.clone().into_iter().map(Box::new).collect()))],
            ASTNode::FunctionCall {
                function: Box::new(AST::new(Token::Identifier(name), vec![])),
                arguments,
            },
        ))
    }

    fn function_definition(&mut self) -> Result<AST, String> {
        self.eat(Token::FN)?;
        
        let name = if let Some(Token::Identifier(name)) = &self.current_token {
            let name = name.clone();
            self.next();
            Some(name)
        } else {
            None
        };

        let params = self.parameter_list()?;

        let body = self.statement()?;

        Ok(AST::with_node(
            Token::FN,
            vec![],
            ASTNode::FunctionDefinition {
                name,
                params,
                body: Box::new(body),
            },
        ))
    }

    fn parameter_list(&mut self) -> Result<Vec<String>, String> {
        let mut params = self.collect_tuple()?;
        Ok(params.iter().map(|x| match &x.token {Token::Identifier(name) => name.clone(), _ => x.token.clone().to_string()}).collect())
    }

    fn collect_tuple(&mut self) -> Result<Vec<Box<AST>>, String> {
        self.eat(Token::LParen)?;

        let mut children = vec![];

        if self.cur_token_unwrap() != Token::RParen {
            children.push(Box::new(self.statement().unwrap()));
            while self.cur_token_unwrap() == Token::Comma {
                self.next();
                children.push(Box::new(self.statement().unwrap()));
            }
        }

        self.eat(Token::RParen)?;
        Ok(children)
    }

    fn identifier(&mut self) -> Result<String, String> {
        if let Some(Token::Identifier(name)) = &self.current_token {
            let name = name.clone();
            self.next();
            Ok(name)
        } else {
            Err("Expected identifier".to_string())
        }
    }
}
