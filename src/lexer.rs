use crate::Token;
use log::error;

pub struct Lexer {
	pub text: String,
	pos: usize,
	current_char: Option<char>
}

impl Lexer {
	pub fn new(text: String) -> Self {
		let mut lexer = Self {
			text,
			pos: 0,
			current_char: None
		};

		lexer.current_char = lexer.text.chars().next();

		lexer
	}

	pub fn reset(&mut self, text: String) {
		*self = Self {
			text: text,
			pos: 0,
			current_char: None
		};
		self.current_char = self.text.chars().next();
	}

	fn error(&self) -> ! {
		error!("Invalid character!");
		panic!("Invalid character!");
	}

	fn advance(&mut self) {
		self.pos += 1;
		self.current_char = self.text.chars().nth(self.pos);
	}

	fn skip_whitespace(&mut self) {
		while let Some(ch) = self.current_char {
			if !ch.is_whitespace() {
				break;
			}
			self.advance();
		}
	}

	fn number(&mut self) -> f64 {
		let mut result = String::new();
		while let Some(ch) = self.current_char {
			if ch.is_digit(10) || ch == '.' {
				result.push(ch);
				self.advance();
			} else {
				break;
			}
		}
		result.parse().unwrap_or_else(|_| self.error())
	}
	
	fn identifier(&mut self) -> String {
		let mut result = String::new();
		while let Some(ch) = self.current_char {
			if ch.is_alphanumeric() || ch == '_' {
				result.push(ch);
				self.advance();
			} else {
				break;
			}
		}
		result
	}

	pub fn get_next_token(&mut self) -> Token {
		self.skip_whitespace();

		match self.current_char {
			Some('f') if self.peek(1) == Some('n') && (self.peek(2).map_or(true, |c| !c.is_alphanumeric())) => {
				self.advance();
				self.advance();
				Token::FN
			},
			Some(',') => { self.advance(); Token::COMMA },
			Some('{') => { self.advance(); Token::LBRACE },
			Some('}') => { self.advance(); Token::RBRACE },

			Some(ch) if ch.is_alphabetic() || ch == '_' => {
				let id = self.identifier();
				Token::IDENTIFIER(id)
			},

			Some(ch) if ch.is_digit(10) || ch == '.' => Token::FLOAT(self.number()),

			Some('+') => { self.advance(); Token::PLUS },
			Some('-') => { self.advance(); Token::MINUS },

			Some('*') => { self.advance(); Token::MUL },
			Some('/') => { self.advance(); Token::DIV },
			Some('%') => { self.advance(); Token::MOD },

			Some('(') => { self.advance(); Token::LPAREN },
			Some(')') => { self.advance(); Token::RPAREN },
			Some(';') => { self.advance(); Token::SEMICOLON },

			Some('&') => {
				self.advance();
				if self.current_char == Some('&') {
					self.advance();
					Token::AND
				} else {
					self.error()
				}
			},
			Some('|') => {
				self.advance();
				if self.current_char == Some('|') {
					self.advance();
					Token::OR
				} else {
					self.error()
				}
			},
			Some('>') => {
				self.advance();
				if self.current_char == Some('=') {
					self.advance();
					Token::GREATER_EQUAL
				} else {
					Token::GREATER
				}
			},
			Some('<') => {
				self.advance();
				if self.current_char == Some('=') {
					self.advance();
					Token::LESS_EQUAL
				} else {
					Token::LESS
				}
			},
			Some('=') => {
				self.advance();
				if self.current_char == Some('=') {
					self.advance();
					Token::EQUAL
				} else {
					Token::ASSIGN
				}
			},
			Some('!') => {
				self.advance();
				if self.current_char == Some('=') {
					self.advance();
					Token::UNEQUAL
				} else {
					Token::NOT
				}
			},
			
			None => Token::EOF,
			_ => self.error(),
		}
	}

	fn peek(&self, offset: usize) -> Option<char> {
		self.text.chars().nth(self.pos + offset)
	}
}