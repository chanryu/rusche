//! Companion example for docs/tutorials/embedding.md

use rusche::{tokenize, Evaluator, Expr, LexError, ParseError, Parser};

fn main() {
    // One expression: tokenize → parse → eval
    let source = "(+ 1 (% 9 2))";
    let tokens = tokenize(source, None).unwrap();
    let mut parser = Parser::with_tokens(tokens);
    let expr = parser.parse().unwrap().unwrap();

    let evaluator = Evaluator::default();
    let result = evaluator.eval(&expr).unwrap();
    assert_eq!(result, Expr::from(2));

    // Inject a host value, then call a built-in
    evaluator.root_env().define("greeting", Expr::from("hello"));
    let tokens = tokenize("(str-append greeting \" world\")", None).unwrap();
    let expr = Parser::with_tokens(tokens).parse().unwrap().unwrap();
    let result = evaluator.eval(&expr).unwrap();
    assert_eq!(result, Expr::from("hello world"));

    // Multi-expression script
    eval_script(
        &evaluator,
        r#"
        (define n 0)
        (set! n (+ n 1))
        (set! n (+ n 1))
        "#,
    )
    .unwrap();
    assert_eq!(
        evaluator
            .eval(&Parser::with_tokens(tokenize("n", None).unwrap()).parse().unwrap().unwrap())
            .unwrap(),
        Expr::from(2)
    );

    println!("{}", describe(&result));
}

fn eval_script(evaluator: &Evaluator, source: &str) -> Result<(), String> {
    let tokens = tokenize(source, None).map_err(|e| match e {
        LexError::InvalidNumber(span) => format!("invalid number at {span}"),
        LexError::IncompleteString(span) => format!("incomplete string at {span}"),
    })?;

    let mut parser = Parser::with_tokens(tokens);
    loop {
        match parser.parse() {
            Ok(None) => return Ok(()),
            Ok(Some(expr)) => {
                evaluator.eval(&expr).map_err(|e| e.message)?;
            }
            Err(ParseError::IncompleteExpr(token)) => {
                return Err(format!("incomplete expression starting at {}", token.span()));
            }
            Err(ParseError::UnexpectedToken(token)) => {
                return Err(format!("unexpected token at {}", token.span()));
            }
        }
    }
}

fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Num(n, _) => format!("number {n}"),
        Expr::Str(s, _) => format!("string {s:?}"),
        Expr::Sym(name, _) => format!("symbol {name}"),
        Expr::List(list, _) => format!("list {list}"),
        Expr::Proc(proc, _) => format!("procedure <{}>", proc.fingerprint()),
        Expr::Foreign(object) => format!("foreign {:p}", object),
    }
}
