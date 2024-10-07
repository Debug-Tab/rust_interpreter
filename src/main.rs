use log::{error, debug};
use std::io::{self, Write};
use env_logger::Env;
use std::error::Error;

mod token;
mod lexer;
mod ast;
mod parser;
mod interpreter;
mod function;
mod test;

use token::{Token, Value, ControlFlow};
use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;


fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let lexer = Lexer::new(String::new());
    let parser = Parser::new(lexer);
    let mut interpreter = Interpreter::new(parser);

    loop {
        let mut text = String::new();

        print!("> ");
        io::stdout().flush()?;

        io::stdin().read_line(&mut text)?;
        
        if text.trim().is_empty() {
            continue;
        }

        match run_interpreter(&mut interpreter, &text) {
            Ok(result) => println!("{}", result),
            Err(e) => error!("Error: {}", e),
        }
    }
}

fn run_interpreter(interpreter: &mut Interpreter, text: &str) -> Result<String, String> {
    interpreter.parser.lexer.reset(text.to_string());
    interpreter.parser.current_token = Some(interpreter.parser.lexer.get_next_token());
    let result = interpreter.interpret()?;
    match result {
        Value::Number(n) => Ok(n.to_string()),
        Value::Function(_) => Ok("Function".to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::Null => Ok("Null".to_string()),
    }
}