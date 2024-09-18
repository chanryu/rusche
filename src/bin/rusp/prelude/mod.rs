mod native;

use std::rc::Rc;

use crate::tokenize::tokenize;

use rusp::env::Env;
use rusp::eval::{eval, EvalContext};
use rusp::expr::Expr;
use rusp::parser::{ParseError, Parser};
use rusp::proc::Proc;

const PRELUDE_SYMBOLS: [&str; 3] = [
    // #t
    "(define #t 1)",
    // #f
    "(define #f '())",
    // numeric operators
    r#"
    (define + num-add)
    (define - num-subtract)
    (define * num-multiply)
    (define / num-divide)
    "#,
];

const PRELUDE_MACROS: [&str; 5] = [
    // if
    r#"
    (defmacro if (pred then else)
        `(cond (,pred ,then)
               (#t    ,else)))
    "#,
    // list
    r#"
    (defmacro list (*args)
        (if (null? args)
            '()
            `(cons ,(car args) (list ,@(cdr args)))))
    "#,
    // let
    r#"
    (defmacro let (bindings *body)
        `((lambda ,(map car bindings) ; Get the list of variable names
             ,@body)                  ; The body of the let becomes the lambda's body
          ,@(map cdar bindings)))     ; Apply the values to the lambda
    "#,
    // begin
    r#"
    (defmacro begin (*exprs)
        `(let () ,@exprs))
    "#,
    // print, println
    r#"
    (defmacro (print *args)
        (if (null? args)
            '()
            `(begin
                (display ,(car args))
                (print ,@(cdr args))
            )
        )
    )
    (defmacro (println *args)
        `(print ,@args)
        `(print "\n"))
    "#,
];

const PRELUDE_FUNCS: [&str; 9] = [
    // caar, cadr, cdar, cdar
    r#"
    (define (caar lst) (car (car lst)))
    (define (cadr lst) (cdr (car lst)))
    (define (cdar lst) (car (cdr lst)))
    (define (cddr lst) (cdr (cdr lst)))
    "#,
    // and, or, not
    r#"
    (define (and x y) (if x (if y #t #f) #f))
    (define (or  x y) (if x #t (if y #t #f)))
    (define (not x  ) (if x #f #t))
    "#,
    // null?
    r#"
    (define (null? e) (eq? e '()))
    "#,
    // map
    r#"
    (define (map fn lst)
        (if (null? lst)
            '()                          ; Base case: empty list
            (cons (fn (car lst))         ; Apply function to the first element
                  (map fn (cdr lst)))))  ; Recursive call on the rest of the list
    "#,
    // append
    r#"
    (define (append lst1 lst2)
        (if (null? lst1) lst2                             ; If lst1 is empty, return lst2
            (cons (car lst1) (append (cdr lst1) lst2))))  ; Otherwise, prepend the first element of lst1 and recurse
    "#,
    // pair
    r#"
    (define (pair lst1 lst2)
        (cond ((and (null? lst1) (null? lst2)) '())
              ((and (not (atom? lst1)) (not (atom? lst2)))
               (cons (cons (car lst1) (cons (car lst2) '()))
                     (pair (cdr lst1) (cdr lst2))))))
    "#,
    // assoc
    r#"
    (define (assoc key lst)
        (cond
            ((null? lst) #f)                       ; If the list is empty, return #f
            ((eq? (car (car lst)) key) (car lst))  ; If the car of the first element matches the key, return the pair
            (#t (assoc key (cdr lst)))))           ; Otherwise, recursively search the rest of the list
    "#,
    // subst
    r#"
    (define (subst new old lst)
        (cond
            ((null? lst) '())                                  ; If the list is empty, return an empty list
            ((eq? (car lst) old)                               ; If the first element matches 'old'
            (cons new (subst new old (cdr lst))))              ; Replace it with 'new' and recurse on the rest
            (#t (cons (car lst) (subst new old (cdr lst))))))  ; Otherwise, keep the first element and recurse
    "#,
    // reverse
    r#"
    (define (reverse lst)
        (if (null? lst) lst
            (append (reverse (cdr lst)) (list (car lst)))))
    "#,
];

pub trait PreludeLoader {
    fn with_prelude() -> Self;
}

impl PreludeLoader for EvalContext {
    fn with_prelude() -> Self {
        let context = Self::new();
        let env = context.root_env();

        env.define(
            "display",
            Expr::Proc(Proc::Native {
                name: "display".to_owned(),
                func: native::display,
            }),
        );

        for exprs in PRELUDE_SYMBOLS {
            eval_prelude_str(exprs, env);
        }
        for exprs in PRELUDE_MACROS {
            eval_prelude_str(exprs, env);
        }
        for exprs in PRELUDE_FUNCS {
            eval_prelude_str(exprs, env);
        }

        context
    }
}

fn eval_prelude_str(text: &str, env: &Rc<Env>) {
    let tokens = tokenize(text).expect(format!("Failed to tokenize prelude: {text}").as_str());

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(expr) => {
                let _ = eval(&expr, &env)
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
#[cfg(test)]
mod tests {
    use super::*;
    use rusp::eval::EvalContext;

    fn eval_str(text: &str) -> String {
        let context = EvalContext::with_prelude();
        eval_str_env(text, context.root_env())
    }

    fn eval_str_env(text: &str, env: &Rc<Env>) -> String {
        let tokens = tokenize(text).expect(&format!("Failed to tokenize: {}", text));
        let mut parser = Parser::with_tokens(tokens);
        let expr = parser
            .parse()
            .expect(&format!("Failed to parse an expression: {}", text));
        if parser.is_parsing() {
            panic!("Too many tokens: {}", text);
        }
        eval(&expr, env)
            .expect(&format!("Failed to evaluate: {}", expr))
            .to_string()
    }

    #[test]
    fn test_t_f() {
        assert_eq!(eval_str("#t"), "1");
        assert_eq!(eval_str("#f"), "()");
    }

    #[test]
    fn test_cxxr() {
        assert_eq!(eval_str("(caar '((1 2) 3 4))"), "1");
        assert_eq!(eval_str("(cadr '((1 2) 3 4))"), "(2)");
        assert_eq!(eval_str("(cdar '((1 2) 3 4))"), "3");
        assert_eq!(eval_str("(cddr '((1 2) 3 4))"), "(4)");
    }

    #[test]
    fn test_if() {
        assert_eq!(eval_str("(if #t 123 456)"), "123");
        assert_eq!(eval_str("(if #f 123 456)"), "456");
        assert_eq!(eval_str("(if 1 (+ 1 2) (+ 3 4))"), "3");
        assert_eq!(eval_str("(if '() (+ 1 2) (+ 3 4))"), "7");
    }

    #[test]
    fn test_list() {
        assert_eq!(eval_str("(list)"), "()");
        assert_eq!(eval_str("(list 1)"), "(1)");
        assert_eq!(eval_str("(list 1 2 3)"), "(1 2 3)");
        assert_eq!(eval_str("(list 1 '(2 3))"), "(1 (2 3))");
    }

    #[test]
    fn test_map() {
        assert_eq!(eval_str("(map (lambda (x) (* x 2)) '(1 2 3))"), "(2 4 6)");
    }

    #[test]
    fn test_let() {
        let context = EvalContext::with_prelude();
        let env = context.root_env();

        assert_eq!(env.lookup("x"), None);
        assert_eq!(eval_str_env("(let ((x 2)) (+ x 3))", env), "5");
        assert_eq!(env.lookup("x"), None);
    }

    #[test]
    fn test_and_or_not() {
        assert_eq!(eval_str("(and #f #f)"), "()");
        assert_eq!(eval_str("(and #f #t)"), "()");
        assert_eq!(eval_str("(and #t #f)"), "()");
        assert_eq!(eval_str("(and #t #t)"), "1");

        assert_eq!(eval_str("(or #f #f)"), "()");
        assert_eq!(eval_str("(or #f #t)"), "1");
        assert_eq!(eval_str("(or #t #f)"), "1");
        assert_eq!(eval_str("(or #t #t)"), "1");

        assert_eq!(eval_str("(not #f)"), "1");
        assert_eq!(eval_str("(not #t)"), "()");
    }

    #[test]
    fn test_append() {
        assert_eq!(eval_str("(append '() '(1))"), "(1)");
        assert_eq!(eval_str("(append '(1) '(2))"), "(1 2)");
        assert_eq!(eval_str("(append '(1 2 3) '(4))"), "(1 2 3 4)");
        assert_eq!(eval_str("(append '(1 2 3) '(4 5 6))"), "(1 2 3 4 5 6)");
    }

    #[test]
    fn test_pair() {
        assert_eq!(
            eval_str(
                r#"(pair '(1 2 3)
                     '("one" "two" "three"))
            "#
            ),
            r#"((1 "one") (2 "two") (3 "three"))"#,
        );

        assert_eq!(eval_str("(pair '(1 2 3 4) '(5 6))"), "((1 5) (2 6))",);
    }

    #[test]
    fn test_assoc() {
        assert_eq!(eval_str("(assoc 'a '((a 1) (b 2) (c 3)))"), "(a 1)");
        assert_eq!(eval_str("(assoc 'b '((a 1) (b 2) (c 3)))"), "(b 2)");
        assert_eq!(eval_str("(assoc 'x '((a 1) (b 2) (c 3)))"), "()");
    }

    #[test]
    fn test_subst() {
        assert_eq!(eval_str("(subst 'a 'b '(a b c b))"), "(a a c a)");
    }

    #[test]
    fn test_reverse() {
        assert_eq!(eval_str("(reverse '(a b c d))"), "(d c b a)");
    }
}
