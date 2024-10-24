use crate::Token;
use log::error;

pub struct Lexer {
	pub tokens: Vec<Token>,
	pos: usize,
}

impl Lexer {
	pub fn new(text: String) -> Self {
		let mut lexer = Self {
			tokens: Vec::new(),
			pos: 0,
		};

		lexer.tokens = lexer.tokenize(text);
		lexer.pos = 0;

		lexer
	}

	pub fn reset(&mut self, text: String) {
		self.pos = 0;
		self.tokens = self.tokenize(text);
		self.pos = 0;
	}

	fn error(&self) -> ! {
		error!("Invalid character!");
		panic!("Invalid character!");
	}

	fn tokenize(&mut self, text: String) -> Vec<Token> {
		let mut tokens = Vec::new();
		let mut current_char = text.chars().peekable();

		while let Some(&ch) = current_char.peek() {
			if ch.is_whitespace() {
				current_char.next();
				continue;
			}
			println!("ch: {}", ch);

			match ch {
				',' => { tokens.push(Token::COMMA); current_char.next(); },
				'{' => { tokens.push(Token::LBRACE); current_char.next(); },
				'}' => { tokens.push(Token::RBRACE); current_char.next(); },
				ch if ch.is_digit(10) || ch == '.' => {
					tokens.push(Token::FLOAT(self.number(&mut current_char)));
				},
				ch if ch.is_alphabetic() || ch == '_' => {
					let id = self.identifier(&mut current_char);
					match id.as_str() {
						"fn" => tokens.push(Token::FN),
						_ => tokens.push(Token::IDENTIFIER(id)),
					}
				},
				'+' => { tokens.push(Token::PLUS); current_char.next(); },
				'-' => { tokens.push(Token::MINUS); current_char.next(); },
				'*' => { tokens.push(Token::MUL); current_char.next(); },
				'/' => { tokens.push(Token::DIV); current_char.next(); },
				'%' => { tokens.push(Token::MOD); current_char.next(); },
				'(' => { tokens.push(Token::LPAREN); current_char.next(); },
				')' => { tokens.push(Token::RPAREN); current_char.next(); },
				';' => { tokens.push(Token::SEMICOLON); current_char.next(); },
				'&' => {
					current_char.next();
					if current_char.peek() == Some(&'&') {
						tokens.push(Token::AND);
						current_char.next();
					} else {
						self.error();
					}
				},
				'|' => {
                    current_char.next();
					if current_char.peek() == Some(&'|') {
						tokens.push(Token::OR);
						current_char.next();
					} else {
						self.error();
					}
                },
				'>' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::GREATER_EQUAL);
						current_char.next();
					} else {
						tokens.push(Token::GREATER);
					}
				},
				'<' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::LESS_EQUAL);
						current_char.next();
					} else {
						tokens.push(Token::LESS);
					}
				},
				'=' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::EQUAL);
						current_char.next();
					} else {
						tokens.push(Token::ASSIGN);
					}
				},
				'!' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::UNEQUAL);
						current_char.next();
					} else {
						tokens.push(Token::NOT);
					}
				},
				_ => self.error(),
			}
		}
		tokens.push(Token::EOF);
		tokens
	}

	fn number(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> f64 {
		let mut result = String::new();
		while let Some(&ch) = chars.peek() {
			if ch.is_digit(10) || ch == '.' {
				result.push(ch);
				chars.next();
			} else {
				break;
			}
		}
		result.parse().unwrap_or_else(|_| self.error())
	}

	fn identifier(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
		let mut result = String::new();
		while let Some(&ch) = chars.peek() {
			if ch.is_alphanumeric() || ch == '_' {
				result.push(ch);
				chars.next();
			} else {
				break;
			}
		}
		result
	}

	pub fn get_next_token(&mut self) -> Token {
		if self.pos < self.tokens.len() {
			let token = self.tokens[self.pos].clone();
			self.pos += 1;
			token
		} else {
			Token::EOF
		}
	}

}
