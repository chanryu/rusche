// Let's prove that the demo code in README.md works.

fn main() {
    use rusche::{tokenize, Evaluator, Expr, Parser};

    let source = "(+ 1 (% 9 2))"; // 1 + (9 % 2) = 1 + 1 = 2

    // Create Evaluator with basic primitives
    let evaluator = Evaluator::with_prelude();

    let mut parser = Parser::new();

    // Tokenize source and add tokens to parser
    let tokens = tokenize(source, None).unwrap();
    parser.add_tokens(tokens);

    // Parse tokens into an expression
    let expr = parser.parse().unwrap().unwrap();

    // Evaluate the parsed expression
    let result = evaluator.eval(&expr);

    assert_eq!(result, Ok(Expr::from(2)));

    println!("{}", result.unwrap()); // this prints out 2
}
