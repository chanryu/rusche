use crate::built_in;
use crate::expr::Expr;
use crate::list::List;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Env {
    vars: RefCell<HashMap<String, Expr>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_root_env() -> Self {
        let env = Env::new();

        // lisp built_in
        env.set("atom", Expr::Proc(built_in::atom));
        env.set("car", Expr::Proc(built_in::car));
        env.set("cdr", Expr::Proc(built_in::cdr));
        env.set("cond", Expr::Proc(built_in::cond));
        env.set("define", Expr::Proc(built_in::define));
        env.set("eq", Expr::Proc(built_in::eq));
        env.set("quote", Expr::Proc(built_in::quote));

        // arithmetic operations
        env.set("+", Expr::Proc(built_in::num::add));
        env.set("-", Expr::Proc(built_in::num::minus));
        env.set("*", Expr::Proc(built_in::num::multiply));
        env.set("/", Expr::Proc(built_in::num::divide));

        env
    }

    pub fn get(&self, name: &str) -> Option<Expr> {
        self.vars.borrow().get(name).cloned()
    }

    pub fn set(&self, name: &str, expr: Expr) {
        self.vars.borrow_mut().insert(name.to_string(), expr);
    }
}

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Sym(name) => match env.get(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(List::Cons(cons)) => {
            if let Expr::Proc(func) = eval(&cons.car, env)? {
                func(cons.cdr.as_ref(), env)
            } else {
                Err(format!("{} does not evaluate to a callable.", cons.car))
            }
        }
        _ => Ok(expr.clone()),
    }
}
