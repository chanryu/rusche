use super::make_syntax_error;
use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

pub fn quote(args: &List, _env: &Env) -> EvalResult {
    let List::Cons(cons) = args else {
        return Err(make_syntax_error("quote", args));
    };

    if !cons.cdr.is_nil() {
        return Err(make_syntax_error("quote", args));
    }

    Ok(cons.car.as_ref().clone())
}

pub fn quasiquote(args: &List, env: &Env) -> EvalResult {
    let List::Cons(cons) = args else {
        return Err(make_syntax_error("quasiquote", args));
    };

    if !cons.cdr.is_nil() {
        return Err(make_syntax_error("quasiquote", args));
    }

    let Expr::List(list) = cons.car.as_ref() else {
        return Ok(cons.car.as_ref().clone());
    };

    let mut exprs = Vec::new();
    let mut iter = list.iter();

    while let Some(expr) = iter.next() {
        let Expr::List(list) = expr else {
            exprs.push(expr.clone());
            continue;
        };

        let List::Cons(cons) = list else {
            exprs.push(List::Nil.into());
            continue;
        };

        let Expr::Sym(name) = cons.car.as_ref() else {
            exprs.push(quasiquote(list, env)?);
            continue;
        };

        match name.as_str() {
            "quote" => {
                exprs.push(expr.clone());
            }
            "unquote" => {
                exprs.push(eval(expr, env)?);
            }
            "unquote-splicing" => {
                let result = eval(expr, env)?;
                if let Expr::List(List::Cons(cons)) = result {
                    exprs.push(cons.car.as_ref().clone());
                    let mut l = cons.cdr.as_ref();
                    while let List::Cons(cons) = l {
                        exprs.push(cons.car.as_ref().clone());
                        l = cons.cdr.as_ref();
                    }
                } else {
                    exprs.push(result);
                }
            }
            _ => {
                exprs.push(quasiquote(list, env)?);
            }
        }
    }

    Ok(exprs.into())
}

pub fn unquote(_args: &List, _env: &Env) -> EvalResult {
    Err("unquote (,) used outside of quasiquote".to_string())
}

pub fn unquote_splicing(_args: &List, _env: &Env) -> EvalResult {
    Err("unquote-splicing (,@) used outside of quasiquote".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::num;
    use crate::list::cons;
    use crate::macros::list;

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2)) => (1 2)
        let result = quote(&list!(list!(num(1), num(2))), &env);
        assert_eq!(result, Ok(list!(num(1), num(2)).into()));
    }

    #[test]
    fn test_quasiquote() {
        let env = Env::new();

        // (quasiquote (0 1 2)) => (0 1 2)
        let result = quasiquote(&list!(list!(num(0), num(1), num(2))), &env);
        assert_eq!(result, Ok(list!(num(0), num(1), num(2)).into()));

        // > (quasiquote (0 (unquote (+ 1 2)) 4))
        // (0 3 4)
    }
}
