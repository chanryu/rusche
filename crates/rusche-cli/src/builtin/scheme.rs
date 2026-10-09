use rusche::Evaluator;

/// Scheme-style names for the core procedures, so scripts written for `rusche-cli` read like
/// Scheme. These are plain aliases defined in the root environment; the core language keeps
/// its own, shorter names.
const SCHEME_ALIASES: &str = r#"
    (define number? num?)
    (define modulo %)
    (define string->number str->num)
    (define number->string num->str)

    (define string? str?)
    (define string-append str-append)
    (define string-compare str-compare)
    (define string-length str-length)
    (define substring str-slice)

    (define symbol? sym?)
    (define procedure? proc?)
    (define symbol->string sym->str)
    (define string->symbol str->sym)

    (define (pair? x) (not (atom? x)))
"#;

pub fn load_scheme_aliases(evaluator: &Evaluator) {
    evaluator
        .eval_str(SCHEME_ALIASES)
        .expect("scheme aliases should evaluate");
}
