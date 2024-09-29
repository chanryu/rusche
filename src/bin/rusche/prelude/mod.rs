mod native;

use rusche::eval::{eval, EvalContext, Evaluator};
use rusche::lexer::tokenize;
use rusche::parser::{ParseError, Parser};

pub trait PreludeLoader {
    fn with_prelude() -> Self;
}

impl PreludeLoader for Evaluator {
    fn with_prelude() -> Self {
        let evaulator = Self::with_builtin();
        let context = evaulator.context();

        context.env.define_native_proc("print", native::print);
        context.env.define_native_proc("read", native::read);

        eval_prelude_str(
            r#"
            (define (read-num) (num-parse (read)))
            (define (println *args) (print *args "\n"))
            "#,
            context,
        );

        evaulator
    }
}

fn eval_prelude_str(text: &str, context: &EvalContext) {
    let tokens = tokenize(text).expect(format!("Failed to tokenize prelude: {text}").as_str());

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(expr) => {
                let _ = eval(&expr, context)
                    .expect(format!("Failed to evaluate prelude: {text}").as_str());
            }
            Err(ParseError::NeedMoreToken) => {
                if parser.is_parsing() {
                    panic!("Failed to parse prelude - incomplete expression: {text}");
                } else {
                    break; // we're done!
                }
            }
            Err(ParseError::UnexpectedToken(token)) => {
                panic!("Failed to parse prelude - unexpected token {token} in {text}");
            }
        }
    }
}
