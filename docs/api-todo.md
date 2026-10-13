# Embedding API TODO

Prioritized checklist for improving the API that hosts use to embed Rusche
(`Evaluator`, `Env`, `Expr`, `Proc`, `Foreign`, `utils`). Ordered by how much
each item improves the embedding experience; items within a group can land
together.

Evidence comes from the two example hosts: `crates/rusche-cli` and
`crates/rusche-turtle`. Both work around the same gaps.

## P1 — Stateful native functions

Done. `NativeFunc` wraps an `Rc<dyn Fn(...)>`, and `define_native_proc` accepts
any `'static` `Fn`. `rusche-turtle` closes over its `Host`. `rusche-cli` keeps
the command line, the output column, and the `load` directory stack on an
`Rc<Host>` captured by the natives that need them.

- [x] Change `Proc::Native.func` from a fn pointer to `NativeFunc` (`Rc<dyn Fn(&str, &List, &EvalContext) -> EvalResult>`)
- [x] Accept `impl Fn(...) + 'static` in `Env::define_native_proc` (plain `fn` items still satisfy `Fn`, so existing registrations keep compiling)
- [x] Switch `PartialEq` and `fingerprint` for natives from `fn_addr_eq` to `Rc` pointer identity
- [x] Migrate `rusche-turtle` off `with_host` / `with_turtle` and `rusche-cli` off its host statics
- [x] Update the "Limits of `NativeFunc`" section in `docs/tutorials/native-functions.md`

## P2 — Call Scheme procedures from Rust

Done. `Evaluator::call` and `EvalContext::call` invoke a procedure with
already-evaluated arguments. `eval_into_proc` checks that an argument is a
procedure. Closures and natives receive the values quoted so they are not
evaluated again; macros receive them as forms, matching `(apply ...)`.

- [x] Add `Evaluator::call(&self, proc: &Expr, args: &[Expr]) -> EvalResult`
- [x] Add `EvalContext::call(...)` for use inside native functions
- [x] Add `eval_into_proc` in `utils` (there are helpers for numbers, strings, ints, and foreign objects, but none for procedures, booleans, symbols, or lists)
- [x] Cover it in `docs/tutorials/embedding.md` with a worked example

## P3 — Interrupt and cancellation

A host can only stop a running script when the script calls one of its natives.
`rusche-turtle` checks its cancel flag inside `random` and notes that tight
loops must call it; a `(while true)` with no native calls is unstoppable.
`Shared` is already consulted on every evaluation step, so the check is cheap.

- [ ] Add `Evaluator::set_interrupt` taking an `Arc<AtomicBool>` (or a step hook), checked every N iterations of the `eval` loop
- [ ] Add an `ErrorKind::Interrupted`
- [ ] Use the same hook for an optional step budget, so a host can sandbox runaway scripts
- [ ] Rewrite the turtle host's cancel path to set the flag instead of checking it per native

## P4 — Value conversion ergonomics

Reading a result means matching tuple variants that carry a span slot
(`Expr::Num(n, _)`). The embedding tutorial's `describe`, the turtle host's
`format_write_args`, and `Key::from_expr` in `rusche-cli`'s dict builtin all
repeat this. Additive and non-breaking; can land incrementally.

- [ ] Add accessors: `Expr::as_num`, `as_bool`, `as_str`, `as_sym`, `as_list`, `as_proc`, and `as_foreign::<T> -> Option<Rc<T>>`
- [ ] Add `Expr::foreign(value)` so hosts stop writing `Expr::Foreign(Rc::new(...))`
- [ ] Add `From` impls for `i64`, `u32`, and `usize`, and `FromIterator<Expr>` for `List` and `Expr`
- [ ] Add `eval_into_bool`, `eval_into_sym`, `eval_into_list`, and `eval_into_proc`
- [ ] Widen integer conversion: `eval_into_int` returns `i32`, so index code such as `vec_get` range-checks by hand; offer an `i64` or `usize` variant
- [ ] Add a variadic arity helper (`get_exact_N` stops at 3; `vec_set` hand-rolls its arity check)
- [ ] Add a second registration form for the common case of evaluated arguments, e.g. `define_native_fn("hypot", |args: &[Expr], ctx| ...)`, keeping the unevaluated form for special-form-like natives
- [ ] Make re-exports consistent: `eval_into_num` and `get_exact_3_args` are not at the crate root while `eval_into_int` is, so hosts import from both `rusche` and `rusche::utils`

## P5 — `ForeignObject` trait (breaking, target 0.6.0)

`Foreign` is `Rc<dyn Any>` (`src/expr.rs`). Consequences: `Display` prints
`<foreign: 0x…>`, `eq?` is pointer identity, and garbage-collection tracing is
registered separately from the type via `register_foreign_tracer`. Forgetting it
is silent — the `vec` and `dict` builtins each carry a comment warning about it.

- [ ] Introduce a `ForeignObject` trait with default methods: `type_name`, `fmt`, `trace(&self, &mut dyn FnMut(&Expr))`, `as_any`
- [ ] Store `Rc<dyn ForeignObject>` so tracing lives on the type and `register_foreign_tracer` is no longer required for the common case
- [ ] Use the trait's `fmt` so `write` and the REPL can print `<vec 1 2 3>`
- [ ] Keep `register_foreign_tracer` working, or provide a migration path, and update `docs/tutorials/foreign.md`
- [ ] Breaking change of its own; the `Proc::Native` signature change already landed with P1

## P6 — Extension and builder API

Each host has an ad-hoc `load_*_procs` with an inconsistent signature
(`load_io_procs(&EvalContext, &Rc<Host>)` versus `load_vec_procs(&Evaluator)`). The turtle
host loads its prelude through `eval_str`, which keeps spans, so errors point
into the prelude text; the core prelude avoids this via the `strip_spans` flag
on `eval_source`.

- [ ] Add an `Extension` trait with `fn install(&self, evaluator: &Evaluator)` and `Evaluator::install`
- [ ] Add `Evaluator::eval_prelude(src)` that strips spans the way the core prelude does
- [ ] Add `Evaluator::builder()` collecting `max_call_depth`, `gc_threshold`, prelude on/off, and extensions
- [ ] Convert the `rusche-cli` and `rusche-turtle` loaders to extensions

## P7 — Named sources in spans

`Span` is line and column only (`src/span.rs`); there is no source name, so
`eval_str` cannot tag a file and `load` in `rusche-cli` prints the inner file's
diagnostic itself before rethrowing a terse error.

- [ ] Add a source name to `Span` or to `Frame.call_site`
- [ ] Add `eval_source_named` (or a source-name parameter on `eval_str`) so multi-file traces name each file
- [ ] Simplify `rusche-cli`'s `load` error path once traces carry the file name
