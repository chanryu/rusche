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
    (define (string=? a b) (= (string-compare a b) 0))

    (define symbol? sym?)
    (define procedure? proc?)
    (define symbol->string sym->str)
    (define string->symbol str->sym)

    (define (pair? x) (not (atom? x)))

    (define vector? vec?)
    (define make-vector vec-make)
    (define vector-length vec-length)
    (define vector-ref vec-get)
    (define vector-set! vec-set!)

    (define (list-ref lst n)
        (if (= n 0)
            (car lst)
            (list-ref (cdr lst) (- n 1))))

    (define assq assoc)
"#;

pub fn load_scheme_aliases(evaluator: &Evaluator) {
    evaluator
        .eval_str(SCHEME_ALIASES)
        .expect("scheme aliases should evaluate");
}
