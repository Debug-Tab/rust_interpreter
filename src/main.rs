use log::{error, debug};
use std::io::{self, BufRead, BufReader, Write};
use env_logger::Env;
use std::error::Error;
use std::fs;
use clap::{Parser, Subcommand};

mod token;
mod value;
mod control_flow;
mod ast_node;
mod lexer;
mod parser;
mod interpreter;
mod function;
mod pre_include;
mod test;

use token::Token;
use interpreter::Interpreter;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Loop,

    Execute { input: String },

    Serialize {
        #[arg(required = true)]
        input: String,
        #[arg(short = 's', long = "serialize")]
        output: String,
    },
    
    Deserialize {
        #[arg(required = true)]
        input: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut interpreter = Interpreter::new();
    interpreter.init().expect("Init Error: ");

    let cli = Cli::parse();

    match cli.command {
        Commands::Loop => {
            loop {
                let mut text = String::new();
        
                print!("> ");
                io::stdout().flush()?;
        
                io::stdin().read_line(&mut text)?;
                
                if text.trim().is_empty() {
                    continue;
                }
        
                match interpreter.run_code(text) {
                    Ok(result) => println!("{}", result),
                    Err(e) => error!("Error: {}", e),
                }
            }
        },
        Commands::Execute { input } => {
            let text = std::fs::read_to_string(input)?;

            match interpreter.run_code(text) {
                Ok(result) => println!("{}", result),
                Err(e) => error!("Error: {}", e),
            }

            Ok(())
        },
        Commands::Serialize { input, output } => {
            let text = std::fs::read_to_string(input)?;
            let mut parser = crate::parser::Parser::new();

            if let Err(e) = parser.reset(text) {
                error!("Error: {}", e);
                return Err(e.into());
            }

            match parser.parse() {
                Ok(result) => {
                    let output_file = fs::File::create(output)?;
                    let mut writer = std::io::BufWriter::new(output_file);
                    let bytes = bincode::serialize(&result)?;
                    writer.write_all(&bytes[..])?;
                },
                Err(e) => {
                    error!("Error: {}", e);
                    return Err(e.into());
                }
            }

            Ok(())
        },
        Commands::Deserialize { input } => {
            
            Ok(())
        },

    }

    
}

