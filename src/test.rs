#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::value::Value;

    fn interpret(text: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        interpreter.interpret(text.to_string())
    }

    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10, "{} != {}", a, b);
    }

    #[test]
    fn test_arithmetic_operations() {
        assert_eq!(interpret("3").unwrap(), Value::Number(3.0));
        assert_eq!(interpret("2 + 7 * 4").unwrap(), Value::Number(30.0));
        assert_eq!(interpret("7 - 8 / 4").unwrap(), Value::Number(5.0));
        assert_eq!(interpret("14 + 2 * 3 - 6 / 2").unwrap(), Value::Number(17.0));
        assert_eq!(interpret("10 % 3").unwrap(), Value::Number(1.0));
        assert_eq!(interpret("22 % 5").unwrap(), Value::Number(2.0));
        assert_float_eq(interpret("0.1 + 0.2").unwrap().to_number().unwrap(), 0.3);
    }

    #[test]
    fn test_complex_expressions() {
        assert_eq!(interpret("7 + 3 * (10 / (12 / (3 + 1) - 1))").unwrap(), Value::Number(22.0));
        assert_eq!(interpret("7 + 3 * (10 / (12 / (3 + 1) - 1)) / (2 + 3) - 5 - 3 + (8)").unwrap(), Value::Number(10.0));
        assert_eq!(interpret("7 + (((3 + 2)))").unwrap(), Value::Number(12.0));
    }

    #[test]
    fn test_unary_operations() {
        assert_eq!(interpret("-5 + 3").unwrap(), Value::Number(-2.0));
        assert_eq!(interpret("-5").unwrap(), Value::Number(-5.0));
        assert_eq!(interpret("3 - (-2)").unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_variable_operations() {
        assert_eq!(interpret("let x = 5; x").unwrap(), Value::Number(5.0));
        assert_eq!(interpret("let x = 5; x + 3").unwrap(), Value::Number(8.0));
        assert_eq!(interpret("let x = 5; let y = 3; x * y").unwrap(), Value::Number(15.0));
        assert_eq!(interpret("let x = 5; let y = 3; let z = x + y; z * 2").unwrap(), Value::Number(16.0));
        assert_eq!(interpret("let x = 5; x = 10; x").unwrap(), Value::Number(10.0));
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(interpret("true && true").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("true && false").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("true || false").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("false || false").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("!true").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("!false").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("true && true && true && true && true").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("false || false || true || false || false").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("true && true && false && true && true").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(interpret("1 == 1").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("1 != 2").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("2 > 1").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("1 < 2").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("2 >= 2").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("2 <= 2").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("1 < 2 && 2 < 3 && 3 < 4").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("1 < 2 && 2 < 3 && 3 > 4").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_error_handling() {
        assert!(interpret("a + 5").is_err());
        assert!(interpret("10 / (5 - 5)").is_err());
        assert!(interpret("10 % 0").is_err());
        assert!(interpret("10 *").is_err());
        assert!(interpret("5 = x").is_err());
        assert!(interpret("(2 + 3 * 4").is_err());
    }

    #[test]
    fn test_multiple_statements() {
        assert_eq!(interpret("let x = 5;let y = 10; x + y").unwrap(), Value::Number(15.0));
        assert_eq!(interpret("let x = 3; x = x * 2; x + 1").unwrap(), Value::Number(7.0));
    }

    #[test]
    fn test_precedence() {
        assert_eq!(interpret("2 + 3 * 4").unwrap(), Value::Number(14.0));
        assert_eq!(interpret("(2 + 3) * 4").unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_function_definition_and_call() {
        let program = r#"
            let add;
            add = fn (a, b) {
                a + b
            };
            add(3, 4)
        "#;
        assert_eq!(interpret(program).unwrap(), Value::Number(7.0));
    }

    /*
    #[test]
    fn test_recursive_function() {
        let program = r#"
            fn factorial(n) {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
            factorial(5)
        "#;
        assert_eq!(interpret(program).unwrap(), Value::Number(120.0));
    }
    */
    
    
    #[test]
    fn test_closure() {
        let program = r#"
            let make_adder;
            make_adder = fn (x) {
                fn(y) {
                    x + y
                }
            };
            let add5; 
            add5 = make_adder(5);
            add5(3)
        "#;
        assert_eq!(interpret(program).unwrap(), Value::Number(8.0));
    }
}