// Let's prove that the demo code in README.md works.

fn main() {
    use rusche::{tokenize, Evaluator, Expr, Parser};

    let source = "(+ 1 (% 9 2))"; // 1 + (9 % 2) = 1 + 1 = 2

    // Tokenize source
    let tokens = tokenize(source, None).unwrap();

    // Create Parser with the tokens
    let mut parser = Parser::with_tokens(tokens);

    // Parse tokens into an expression
    let expr = parser.parse().unwrap().unwrap();

    // Create Evaluator with basic primitives
    let evaluator = Evaluator::default();

    // Evaluate the parsed expression
    let result = evaluator.eval(&expr);

    assert_eq!(result, Ok(Expr::from(2)));

    println!("{}", result.unwrap()); // this prints out 2
}
