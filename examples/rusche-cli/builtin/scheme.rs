use rusche::{tokenize, Evaluator, Parser};

/// Scheme-style names for the core procedures, so scripts written for `rusche-cli` read like
/// Scheme. These are plain aliases defined in the root environment; the core language keeps
/// its own, shorter names.
const SCHEME_ALIASES: &str = r#"
    (define number? num?)
    (define modulo %)
    (define string->number num-parse)

    (define string? str?)
    (define string-append str-append)
    (define string-compare str-compare)
    (define string-length str-length)
    (define substring str-slice)

    (define (pair? x) (not (atom? x)))
"#;

pub fn load_scheme_aliases(evaluator: &Evaluator) {
    let tokens = tokenize(SCHEME_ALIASES, None).expect("scheme aliases should tokenize");
    let mut parser = Parser::with_tokens(tokens);
    while let Some(expr) = parser.parse().expect("scheme aliases should parse") {
        evaluator
            .eval(&expr)
            .expect("scheme aliases should evaluate");
    }
}
