use crate::Token;

use serde::{Serialize, Deserialize};
use log::{debug, error};

#[derive(Serialize, Deserialize, Debug)]
pub struct Lexer {
	pub tokens: Vec<Token>,
	pos: usize,
}

impl Lexer {
	pub fn new() -> Self {
		Self {
			tokens: Vec::new(),
			pos: 0,
		}
	}

	fn error(&self) -> ! {
		error!("Invalid character!");
		panic!("Invalid character!");
	}

	pub fn tokenize(text: String) -> Result<Vec<Token>, String> {
		let lexer = Lexer::new();

		let mut tokens = Vec::new();
		let mut current_char = text.chars().peekable();

		while let Some(&ch) = current_char.peek() {
			if ch.is_whitespace() {
				current_char.next();
				continue;
			}

			match ch {
				'"' => {
					tokens.push(Token::String(lexer.string(&mut current_char)?));
				}
				ch if ch.is_digit(10) || ch == '.' => {
					tokens.push(Token::Float(lexer.number(&mut current_char)));
				},
				ch if ch.is_alphabetic() || ch == '_' => {
					let id = lexer.identifier(&mut current_char);
					match id.as_str() {
						"fn" => tokens.push(Token::FN),
						"return" => tokens.push(Token::Return),

						"true" => tokens.push(Token::True),
						"false" => tokens.push(Token::False),
						"null" => tokens.push(Token::Null),

						"let" => tokens.push(Token::Let),
						"if" => tokens.push(Token::If),
						"else" => tokens.push(Token::Else),
						"break" => tokens.push(Token::Break),

						"while" => tokens.push(Token::While),
						_ => tokens.push(Token::Identifier(id)),
					}
				},
				'+' => { tokens.push(Token::Plus); current_char.next(); },
				'-' => { tokens.push(Token::Minus); current_char.next(); },
				'*' => { tokens.push(Token::Mul); current_char.next(); },
				'/' => { tokens.push(Token::Div); current_char.next(); },
				'%' => { tokens.push(Token::Mod); current_char.next(); },
				',' => { tokens.push(Token::Comma); current_char.next(); },
				'(' => { tokens.push(Token::LParen); current_char.next(); },
				')' => { tokens.push(Token::RParen); current_char.next(); },
				'[' => { tokens.push(Token::LBracket); current_char.next(); },
				']' => { tokens.push(Token::RBracket); current_char.next(); },
				'{' => { tokens.push(Token::LBrace); current_char.next(); },
				'}' => { tokens.push(Token::RBrace); current_char.next(); },
				';' => { tokens.push(Token::Semicolon); current_char.next(); },
				'?' => { tokens.push(Token::Question); current_char.next(); },
				':' => { tokens.push(Token::Colon); current_char.next(); },
				'&' => {
					current_char.next();
					if current_char.peek() == Some(&'&') {
						tokens.push(Token::And);
						current_char.next();
					} else {
						lexer.error();
					}
				},
				'|' => {
                    current_char.next();
					if current_char.peek() == Some(&'|') {
						tokens.push(Token::Or);
						current_char.next();
					} else {
						lexer.error();
					}
                },
				'>' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::GreaterEqual);
						current_char.next();
					} else {
						tokens.push(Token::Greater);
					}
				},
				'<' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::LessEqual);
						current_char.next();
					} else {
						tokens.push(Token::Less);
					}
				},
				'=' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::Equal);
						current_char.next();
					} else {
						tokens.push(Token::Assign);
					}
				},
				'!' => {
					current_char.next();
					if current_char.peek() == Some(&'=') {
						tokens.push(Token::UnEqual);
						current_char.next();
					} else {
						tokens.push(Token::Not);
					}
				},
				_ => lexer.error(),
			}
		}
		tokens.push(Token::EOF);
		Ok(tokens)
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

	fn string(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, String> {
		let mut result = String::new();

		chars.next();
		while let Some(&ch) = chars.peek() {
			if ch != '"' {
                match ch {
                    '\\' => {
						chars.next();
						result.push(
						if let Some(ch) = chars.peek() {
								match ch {
									'"' => '"',
									'\\' => '\\',
									'/' => '/',
									'b' => 0x08 as char, // \b
									'f' => 0x0c as char, // \f
									'n' => '\n',
									'r' => '\r',
									't' => '\t',
									'u' => {
										'\0' // todo
									}
									'0' => '\0',
									_ => {
										return Err(format!("Unknown character escape: '\\{}'", ch));
									}
								}
							} else {
								return Err(format!("The string has not ended yet!"));
							}
						);
					},
                    '\n' => return Err(format!("Unexpected string ending: \\n")),
                    _ => result.push(ch)
                };
				chars.next();
			} else {
				break;
			}
		}

		chars.next();
		debug!("String result: {}", result.clone());	
		Ok(result)
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
}
