use rusche::{
    eval::{eval, EvalContext, EvalResult},
    expr::{Expr, NIL},
    lexer::tokenize,
    list::List,
    parser::{ParseError, Parser},
};
use std::io::Write;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("print", print);
    context.env.define_native_proc("read", read);

    eval_prelude_str(
        r#"
            (define (read-num) (num-parse (read)))
            (define (println *args) (print *args "\n"))
            "#,
        context,
    );
}

fn print(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(format!("Error reading input: {}", error));
    }
    Ok(Expr::from(input.trim()))
}

fn eval_prelude_str(text: &str, context: &EvalContext) {
    let tokens = tokenize(text).expect(format!("Failed to tokenize prelude: {text}").as_str());

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(None) => {
                break; // we're done!
            }
            Ok(Some(expr)) => {
                let _ = eval(&expr, context)
                    .expect(format!("Failed to evaluate prelude: {text}").as_str());
            }
            Err(ParseError::NeedMoreToken) => {
                assert!(parser.is_parsing());
                panic!("Failed to parse prelude - incomplete expression: {text}");
            }
            Err(ParseError::UnexpectedToken(token)) => {
                panic!("Failed to parse prelude - unexpected token {token} in {text}");
            }
        }
    }
}
