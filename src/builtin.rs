pub mod quote;

mod convert;
mod list;
mod num;
mod proc;
pub(crate) mod special;
mod str;
mod sym;

use std::rc::Rc;

use crate::env::Env;

pub fn load_builtin(env: &Rc<Env>) {
    // special forms and evaluator ops
    // (`quote`, `quasiquote`, `begin`, `if`, `eval`, and `apply` are handled by the evaluator
    // itself; see `eval::form`.)
    env.define_native_proc("define", special::define);
    env.define_native_proc("defmacro", special::defmacro);
    env.define_native_proc("eq?", special::eq);
    env.define_native_proc("error", special::error);
    env.define_native_proc("lambda", special::lambda);
    env.define_native_proc("set!", special::set);

    // list
    env.define_native_proc("atom?", list::is_atom);
    env.define_native_proc("car", list::car);
    env.define_native_proc("cdr", list::cdr);
    env.define_native_proc("cons", list::cons);
    env.define_native_proc("null?", list::is_null);

    // num
    env.define_native_proc("num?", num::is_num);
    env.define_native_proc("num-add", num::add);
    env.define_native_proc("num-subtract", num::subtract);
    env.define_native_proc("num-multiply", num::multiply);
    env.define_native_proc("num-divide", num::divide);
    env.define_native_proc("num-modulo", num::modulo);
    env.define_native_proc("num-less", num::less);
    env.define_native_proc("num-sqrt", num::sqrt);
    env.define_native_proc("num-exp", num::exp);
    env.define_native_proc("num-log", num::log);
    env.define_native_proc("num-expt", num::expt);

    // str
    env.define_native_proc("str?", str::is_str);
    env.define_native_proc("str-append", str::append);
    env.define_native_proc("str-compare", str::compare);
    env.define_native_proc("str-length", str::length);
    env.define_native_proc("str-slice", str::slice);
    env.define_native_proc("str-find", str::find);
    env.define_native_proc("str-split", str::split);
    env.define_native_proc("str-trim", str::trim);
    env.define_native_proc("str-trim-left", str::trim_left);
    env.define_native_proc("str-trim-right", str::trim_right);
    env.define_native_proc("str-upcase", str::upcase);
    env.define_native_proc("str-downcase", str::downcase);
    env.define_native_proc("str-replace", str::replace);
    env.define_native_proc("str-join", str::join);

    // sym
    env.define_native_proc("sym?", sym::is_sym);

    // proc
    env.define_native_proc("proc?", proc::is_proc);

    // conversions
    env.define_native_proc("num->str", convert::num_to_str);
    env.define_native_proc("str->num", convert::str_to_num);
    env.define_native_proc("sym->str", convert::sym_to_str);
    env.define_native_proc("str->sym", convert::str_to_sym);
}
