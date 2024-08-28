use crate::env::Env;
use crate::eval::EvalResult;
use crate::expr::Expr;
use crate::list::List;

#[derive(Clone, Debug, PartialEq)]
pub enum Proc {
    Native(NativeFunc),
    Closure {
        formal_args: List,
        lambda_body: List,
        outer_env: Env,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, env: &Env) -> EvalResult {
        match self {
            Proc::Native(func) => func(args, env),
            Proc::Closure {
                formal_args,
                lambda_body,
                outer_env,
            } => eval_closure(formal_args, lambda_body, outer_env, args, env),
        }
    }
}

pub type NativeFunc = fn(args: &List, env: &Env) -> EvalResult;

fn eval_closure(
    formal_args: &List,
    lambda_body: &List,
    outer_env: &Env,
    args: &List,
    env: &Env,
) -> EvalResult {
    // auto lambda_env = outer_env->derive_new();
    // auto syms = formal_args;
    // while (!syms.empty()) {
    //     auto sym = dynamic_node_cast<Symbol>(car(syms));
    //     assert(sym.has_value());
    //     if (args.empty()) {
    //         throw EvalError("Proc: too few args");
    //     }
    //     auto val = eval(car(args), env);
    //     lambda_env->set(sym->name(), val);
    //     syms = cdr(syms);
    //     args = cdr(args);
    // }
    // if (!args.empty()) {
    //     throw EvalError("Proc: too many args");
    // }
    // Node result;
    // for_each(lambda_body, [&result, lambda_env](auto const& expr) { result = eval(expr, *lambda_env); });

    Ok(Expr::List(List::Nil))
}
