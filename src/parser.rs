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
        
        while self.current_token != Some(Token::EOF) && self.current_token != Some(Token::RBRACE) {
            let stmt = self.statement()?;
            statements.push(Box::new(stmt));
            
            if let Some(Token::SEMICOLON) = self.current_token {
                self.next();
            } else if self.current_token != Some(Token::EOF) && self.current_token != Some(Token::RBRACE) {
                return Err("Expected semicolon.".to_string());
            }
        }
        
        Ok(AST::new(Token::STATEMENT, statements))
    }

    fn statement(&mut self) -> Result<AST, String> {
        match self.current_token {
            Some(Token::FN) => self.function_definition(),
            Some(Token::LBRACE) => {
                self.next();
                let block = self.statements();
                self.eat(Token::RBRACE)?;
                block
            }
            _ => {
                let mut node = self.expression()?;

                if let Some(Token::ASSIGN) = self.current_token {
                    self.eat(Token::ASSIGN)?;
                    let expr = self.statement()?;
                    node = AST::new(Token::ASSIGN, vec![Box::new(node), Box::new(expr)]);
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

        while let Some(Token::OR) = self.current_token {
            self.next();
            let right = self.logical_and()?;
            node = AST::new(Token::OR, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn logical_and(&mut self) -> Result<AST, String> {
        let mut node = self.equality()?;

        while let Some(Token::AND) = self.current_token {
            self.next();
            let right = self.equality()?;
            node = AST::new(Token::AND, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn equality(&mut self) -> Result<AST, String> {
        let mut node = self.relational()?;

        while let token @ (Token::EQUAL | Token::UNEQUAL) = self.cur_token_unwrap() {
            self.next();
            let right = self.relational()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn relational(&mut self) -> Result<AST, String> {
        let mut node = self.additive_expression()?;

        while let token @ (Token::GREATER | Token::LESS | Token::GREATER_EQUAL | Token::LESS_EQUAL) = self.cur_token_unwrap() {
            self.next();
            let right = self.additive_expression()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn additive_expression(&mut self) -> Result<AST, String> {
        let mut node = self.multiplicative_expression()?;

        while let token @ (Token::PLUS | Token::MINUS) = self.cur_token_unwrap() {
            self.next();
            let right = self.multiplicative_expression()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn multiplicative_expression(&mut self) -> Result<AST, String> {
        let mut node = self.unary_expression()?;

        while let token @ (Token::MUL | Token::DIV | Token::MOD) = self.cur_token_unwrap() {
            self.next();
            let right = self.unary_expression()?;
            node = AST::new(token, vec![Box::new(node), Box::new(right)]);
        }

        Ok(node)
    }

    fn unary_expression(&mut self) -> Result<AST, String> {
        if let token @ (Token::PLUS | Token::MINUS | Token::NOT) = self.cur_token_unwrap() {
            self.next();
            let expr = self.unary_expression()?;
            Ok(AST::new(token, vec![Box::new(expr)]))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<AST, String> {
        match &self.current_token {
            Some(Token::IDENTIFIER(name)) => {
                let name = name.clone();
                self.next();
                if self.current_token == Some(Token::LPAREN) {
                    // 函数调用
                    self.function_call(name)
                } else {
                    Ok(AST::new(Token::IDENTIFIER(name), vec![]))
                }
            },
            Some(Token::FLOAT(value)) => {
                let value = *value;
                self.next();
                Ok(AST::new(Token::FLOAT(value), vec![]))
            },
            Some(Token::LPAREN) => {
                self.next();
                let node = self.expression()?;
                self.eat(Token::RPAREN)?;
                Ok(node)
            },
            _ => Err(format!("Unexpected token: {}!", self.cur_token_unwrap())),
        }
    }

    fn function_call(&mut self, name: String) -> Result<AST, String> {
        self.eat(Token::LPAREN)?;
        let mut arguments = vec![];

        if self.current_token != Some(Token::RPAREN) {
            arguments.push(self.expression()?);
            while self.current_token == Some(Token::COMMA) {
                self.next();
                arguments.push(self.expression()?);
            }
        }

        self.eat(Token::RPAREN)?;

        Ok(AST::with_node(
            Token::CALL,
            vec![Box::new(AST::new(Token::IDENTIFIER(name.clone()), vec![])), 
            Box::new(AST::new(Token::STATEMENT, arguments.clone().into_iter().map(Box::new).collect()))],
            ASTNode::FunctionCall {
                function: Box::new(AST::new(Token::IDENTIFIER(name), vec![])),
                arguments,
            },
        ))
    }

    fn function_definition(&mut self) -> Result<AST, String> {
        self.eat(Token::FN)?;
        
        let name = if let Some(Token::IDENTIFIER(name)) = &self.current_token {
            let name = name.clone();
            self.next();
            Some(name)
        } else {
            None
        };

        self.eat(Token::LPAREN)?;
        let params = self.parameter_list()?;
        self.eat(Token::RPAREN)?;

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
        let mut params = Vec::new();
        if self.current_token != Some(Token::RPAREN) {
            params.push(self.identifier()?);
            while self.current_token == Some(Token::COMMA) {
                self.next();
                params.push(self.identifier()?);
            }
        }
        Ok(params)
    }

    fn identifier(&mut self) -> Result<String, String> {
        if let Some(Token::IDENTIFIER(name)) = &self.current_token {
            let name = name.clone();
            self.next();
            Ok(name)
        } else {
            Err("Expected identifier".to_string())
        }
    }
}
