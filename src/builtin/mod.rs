pub mod quote;
pub mod utils;

mod num;
mod primitive;
mod str;

use std::rc::Rc;

use crate::env::Env;
use utils::{get_exact_1_arg, make_syntax_error};

pub fn load_builtin(env: &Rc<Env>) {
    // lisp primitives
    env.define_native_proc("atom?", primitive::atom);
    env.define_native_proc("car", primitive::car);
    env.define_native_proc("cdr", primitive::cdr);
    env.define_native_proc("cons", primitive::cons);
    env.define_native_proc("define", primitive::define);
    env.define_native_proc("defmacro", primitive::defmacro);
    env.define_native_proc("eq?", primitive::eq);
    env.define_native_proc("eval", primitive::eval_);
    env.define_native_proc("if", primitive::if_);
    env.define_native_proc("lambda", primitive::lambda);
    env.define_native_proc("set!", primitive::set);

    // num
    env.define_native_proc("num?", num::is_num);
    env.define_native_proc("num-add", num::add);
    env.define_native_proc("num-subtract", num::subtract);
    env.define_native_proc("num-multiply", num::multiply);
    env.define_native_proc("num-divide", num::divide);
    env.define_native_proc("num-modulo", num::modulo);
    env.define_native_proc("num-less", num::less);
    env.define_native_proc("num-greater", num::greater);
    env.define_native_proc("num-parse", num::parse);

    // str
    env.define_native_proc("str?", str::is_str);
    env.define_native_proc("str-append", str::append);
    env.define_native_proc("str-compare", str::compare);
    env.define_native_proc("str-length", str::length);
    env.define_native_proc("str-slice", str::slice);
}
