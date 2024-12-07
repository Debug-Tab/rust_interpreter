use crate::value::Value;
use crate::environment::Environment;
use chrono::Utc;

pub fn initialization(env: &mut Environment) -> Result<(), String> {
    env.define("timestamp".to_string(), Value::Hole(1))?;
    env.define("printf".to_string(), Value::Hole(12))?;
    Ok(())
}

pub fn hole_func(id: u32, args: Vec<Value>) -> Result<Value, String> {
    match id {
        1 => {
            Ok(Value::Number(Utc::now().timestamp() as f64))
        }
        12 => {
            if let Value::String(format) = &args[0] {
                let formatted = format_string(format, &args[1..])?;
                print!("{}", formatted);
                Ok(Value::Nothing)
            } else {
                Err(format!("The first argument must be a string, actually found: {}", args[0]))
            }
        },
        _ => Err(format!("No hole func: {}", id)),
    }
}

fn format_string(format: &str, args: &[Value]) -> Result<String, String> {
    let mut result = String::new();
    let mut arg_index = 0;

    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'}') {
                chars.next();
                if arg_index < args.len() {
                    result.push_str(&args[arg_index].to_string());
                    arg_index += 1;
                } else {
                    return Err("Not enough arguments for format string".to_string());
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    if arg_index < args.len() {
        Err(format!("Too many arguments for format string. Need {}, found {}", arg_index, args.len()))
    } else {
        Ok(result)
    }
}