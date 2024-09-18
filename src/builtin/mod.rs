pub mod quote;

mod num;
mod primitive;
mod str;
mod utils;

use std::rc::Rc;

use utils::{get_exact_1_arg, make_syntax_error};

use crate::env::Env;
use crate::expr::Expr;
use crate::proc::Proc;

pub fn load_builtin(env: &Rc<Env>) {
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
    set_native_func("atom?", primitive::atom);
    set_native_func("car", primitive::car);
    set_native_func("cdr", primitive::cdr);
    set_native_func("cons", primitive::cons);
    set_native_func("cond", primitive::cond);
    set_native_func("define", primitive::define);
    set_native_func("defmacro", primitive::defmacro);
    set_native_func("eq?", primitive::eq);
    set_native_func("eval", primitive::eval_);
    set_native_func("lambda", primitive::lambda);
    set_native_func("set!", primitive::set);

    // num
    set_native_func("num?", num::is_num);
    set_native_func("num-add", num::add);
    set_native_func("num-subtract", num::subtract);
    set_native_func("num-multiply", num::multiply);
    set_native_func("num-divide", num::divide);

    // str
    set_native_func("str?", str::is_str);
    set_native_func("str-compare", str::compare);
    set_native_func("str-concat", str::concat);
    set_native_func("str-length", str::length);
    set_native_func("str-slice", str::slice);
}
