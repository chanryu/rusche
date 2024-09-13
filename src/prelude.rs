use crate::built_in;
use crate::env::Env;
use crate::eval::eval;
use crate::expr::Expr;
use crate::parser::{ParseError, Parser};
use crate::proc::Proc;
use crate::scanner::Scanner;

pub fn load_prelude(env: &Env) {
    load_native_functions(env);
    load_complementry_items(env);
}

fn load_native_functions(env: &Env) {
    let set_native_func = |name, func| {
        env.define(
            name,
            Expr::Proc(Proc::Native {
                name: name.to_owned(),
                func,
            }),
        );
    };

    // lisp primitives
    set_native_func("atom?", built_in::atom);
    set_native_func("car", built_in::car);
    set_native_func("cdr", built_in::cdr);
    set_native_func("cons", built_in::cons_);
    set_native_func("cond", built_in::cond);
    set_native_func("define", built_in::define);
    set_native_func("defmacro", built_in::defmacro);
    set_native_func("display", built_in::display);
    set_native_func("eq?", built_in::eq);
    set_native_func("eval", built_in::eval_);
    set_native_func("lambda", built_in::lambda);
    set_native_func("set!", built_in::set);

    // quote
    set_native_func("quote", built_in::quote::quote);
    set_native_func("quasiquote", built_in::quote::quasiquote);

    // num
    set_native_func("+", built_in::num::add);
    set_native_func("-", built_in::num::minus);
    set_native_func("*", built_in::num::multiply);
    set_native_func("/", built_in::num::divide);
    set_native_func("num?", built_in::num::is_num);
}

const COMPLEMENTRY_PRELUDE_SYMBOLS: [&str; 2] = [
    // #t
    "(define #t 1)",
    // #f
    "(define #f '())",
];

const COMPLEMENTRY_PRELUDE_MACROS: [&str; 4] = [
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
];

const COMPLEMENTRY_PRELUDE_FUNCS: [&str; 10] = [
    // caar, cadr, cdar, cdar
    r#"
    (define (caar  lst) (car (car lst)))
    (define (cadr  lst) (cdr (car lst)))
    (define (cdar  lst) (car (cdr lst)))
    (define (cddr  lst) (cdr (cdr lst)))
    "#,
    // cadar
    r#"
    (define (cadar lst) (car (cadr lst)))
    (define (caddr lst) (cdr (cadr lst)))
    "#,
    // map
    r#"
    (define (map fn lst)
        (if (null? lst)
            '()                          ; Base case: empty list
            (cons (fn (car lst))         ; Apply function to the first element
                  (map fn (cdr lst)))))  ; Recursive call on the rest of the list
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

fn load_complementry_items(env: &Env) {
    for exprs in COMPLEMENTRY_PRELUDE_SYMBOLS {
        eval_prelude_exprs(exprs, env);
    }
    for exprs in COMPLEMENTRY_PRELUDE_MACROS {
        eval_prelude_exprs(exprs, env);
    }
    for exprs in COMPLEMENTRY_PRELUDE_FUNCS {
        eval_prelude_exprs(exprs, env);
    }
}

fn eval_prelude_exprs(exprs: &str, env: &Env) {
    let mut scanner = Scanner::new(exprs.chars());
    let tokens = std::iter::from_fn(|| match scanner.get_token() {
        Ok(Some(token)) => Some(token),
        Ok(None) => None,
        Err(_) => panic!("Failed to tokenize prelude: {exprs}"),
    })
    .collect::<Vec<_>>();

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(expr) => {
                let _ = eval(&expr, &env)
                    .expect(format!("Failed to evaluate prelude: {exprs}").as_str());
            }
            Err(ParseError::NeedMoreToken) => {
                if parser.is_parsing() {
                    panic!("Failed to parse prelude - incomplete expression: {exprs}");
                } else {
                    break; // we're done!
                }
            }
            Err(ParseError::UnexpectedToken(token)) => {
                panic!("Failed to parse prelude - unexpected token {token} in {exprs}");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complementry_pieces_sanity() {
        load_prelude(&Env::new()); // this should not panic
    }
}
