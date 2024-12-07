use log::{error, debug};
use std::io::{self, Write};
use env_logger::Env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::ffi::OsStr;
use clap::{Parser, Subcommand};
use chrono::Utc;

mod token;
mod value;
mod control_flow;
mod ast_node;
mod lexer;
mod parser;
mod interpreter;
mod environment;
mod pre_include;
mod test;

use token::Token;
use interpreter::Interpreter;

/// An Interpreter for Lim
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Loop,

    Run { 
        #[arg(required = true)]
        input: String
    },

    Build {
        #[arg(required = true)]
        input: String,
        #[arg(required = false)]
        output: Option<String>,
    },
}

fn input_loop(interpreter: &mut Interpreter) -> Result<(), Box<dyn Error>> {
    print!("Time: {:?}\n", Utc::now());

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
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();

    let mut interpreter = Interpreter::new();
    interpreter.init().expect("Init Error: ");

    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Commands::Loop => {
                input_loop(&mut interpreter)
            },

            Commands::Run { input } => {
                let path = Path::new(&input);

                if path.extension() == Some(OsStr::new("lim")) {
                    let bytes = fs::read(input)?;

                    match interpreter.evaluate(&bincode::deserialize(&bytes[..]).unwrap()) {
                        Ok(result) => println!("{}", result.unwrap()),
                        Err(e) => error!("Error: {}", e),
                    }
                
                } else {
                    let text = std::fs::read_to_string(input)?;

                    match interpreter.run_code(text) {
                        Ok(result) => println!("{}", result),
                        Err(e) => error!("Error: {}", e),
                    }
                }

                Ok(())
            },

            Commands::Build { input, output } => {
                let text = std::fs::read_to_string(&input)?;
                let mut parser = crate::parser::Parser::new();

                if let Err(e) = parser.reset(text) {
                    error!("Error: {}", e);
                    return Err(e.into());
                }

                match parser.parse() {
                    Ok(result) => {
                        let path = match output {
                            Some(path) => {
                                path.clone()
                            },
                            None => {
                                let path = Path::new(&input);
                                let name = path
                                                    .file_stem()
                                                    .expect("No output file name.")
                                                    .to_str()
                                                    .expect("Cannot convert output file name to str.");
                                
                                path
                                    .join("..")
                                    .join(format!("{}.lim", name))
                                    .to_str()
                                    .expect("Cannot convert output path to str.")
                                    .to_string()
                                
                            }
                        };
                        let output_file = fs::File::create(path)?;
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
        }
    } else {
        input_loop(&mut interpreter)
    }
}

