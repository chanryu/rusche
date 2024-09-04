use crate::env::Env;
use crate::eval::eval;
use crate::parser::Parser;
use crate::scanner::Scanner;

const DEFMACRO: &str = r#"
    (define (defmacro name args body)
        (eval `(define ,name
                (lambda ,args (eval ,body)))))
    "#;

pub fn load_prelude(env: &Env) {
    for expr in [DEFMACRO] {
        let mut tokens = Vec::new();
        let mut scanner = Scanner::new(expr.chars());
        while let Some(token) = scanner.get_token().expect("Prelude failure!") {
            tokens.push(token);
        }

        let mut parser = Parser::new();
        parser.add_tokens(tokens);
        let expr = parser.parse().expect("Prelude failure!");
        if parser.is_parsing() {
            panic!();
        }

        let _ = eval(&expr, &env).expect("Prelude failure!");
    }
}
