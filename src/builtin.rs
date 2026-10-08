pub mod quote;

mod convert;
mod num;
mod primitive;
mod str;

use std::rc::Rc;

use crate::env::Env;

pub fn load_builtin(env: &Rc<Env>) {
    // lisp primitives
    env.define_native_proc("apply", primitive::apply);
    env.define_native_proc("car", primitive::car);
    env.define_native_proc("cdr", primitive::cdr);
    env.define_native_proc("cons", primitive::cons);
    env.define_native_proc("define", primitive::define);
    env.define_native_proc("defmacro", primitive::defmacro);
    env.define_native_proc("eq?", primitive::eq);
    env.define_native_proc("error", primitive::error);
    env.define_native_proc("eval", primitive::eval_);
    env.define_native_proc("if", primitive::if_);
    env.define_native_proc("lambda", primitive::lambda);
    env.define_native_proc("atom?", primitive::is_atom);
    env.define_native_proc("proc?", primitive::is_proc);
    env.define_native_proc("sym?", primitive::is_sym);
    env.define_native_proc("set!", primitive::set);

    // num
    env.define_native_proc("num?", num::is_num);
    env.define_native_proc("num-add", num::add);
    env.define_native_proc("num-subtract", num::subtract);
    env.define_native_proc("num-multiply", num::multiply);
    env.define_native_proc("num-divide", num::divide);
    env.define_native_proc("num-modulo", num::modulo);
    env.define_native_proc("num-less", num::less);

    // str
    env.define_native_proc("str?", str::is_str);
    env.define_native_proc("str-append", str::append);
    env.define_native_proc("str-compare", str::compare);
    env.define_native_proc("str-length", str::length);
    env.define_native_proc("str-slice", str::slice);

    // conversions
    env.define_native_proc("num->str", convert::num_to_str);
    env.define_native_proc("str->num", convert::str_to_num);
    env.define_native_proc("sym->str", convert::sym_to_str);
    env.define_native_proc("str->sym", convert::str_to_sym);
}
