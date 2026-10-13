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

Done. `Evaluator::set_interrupt` watches an `Arc<AtomicBool>`, loaded on the
first eval step and every `INTERRUPT_CHECK_INTERVAL` steps after that.
`set_step_budget` counts those same steps and fails a top-level `eval`,
`eval_str`, or `call` that exceeds it. Both report `ErrorKind::Interrupted`.
`rusche-turtle` Stop sets the flag; a `(while true)` with no native calls is
stoppable.

- [x] Add `Evaluator::set_interrupt` taking an `Arc<AtomicBool>` (or a step hook), checked every N iterations of the `eval` loop
- [x] Add an `ErrorKind::Interrupted`
- [x] Use the same hook for an optional step budget, so a host can sandbox runaway scripts
- [x] Rewrite the turtle host's cancel path to set the flag instead of checking it per native

## P4 — Value conversion ergonomics

Partly done. Hosts read values with `Expr::as_num`, `as_bool`, `as_str`,
`as_sym`, `as_list`, `as_proc`, and `as_foreign::<T>`, and build foreign objects
with `Expr::foreign`. `eval_into_list` and `eval_into_usize` cover the list and
index conversions `rusche-cli` was writing by hand. `From<usize>` and
`FromIterator<Expr>` cover lengths and list construction. The conversion and
arity helpers are re-exported from the crate root.

Still open: `From` for `i64` and `u32` (nothing constructs those today),
`eval_into_bool` and `eval_into_sym` (the accessors cover the current readers),
a variadic arity helper (`vec-set!` uses the existing 3-argument helper), and
`define_native_fn` for natives whose arguments are all evaluated.

- [x] Add accessors: `Expr::as_num`, `as_bool`, `as_str`, `as_sym`, `as_list`, `as_proc`, and `as_foreign::<T> -> Option<Rc<T>>`
- [x] Add `Expr::foreign(value)` so hosts stop writing `Expr::Foreign(Rc::new(...))`
- [x] Add `From<usize>` and `FromIterator<Expr>` for `List` and `Expr`
- [ ] Add `From` impls for `i64` and `u32`
- [x] Add `eval_into_list` (`eval_into_proc` landed with P2)
- [ ] Add `eval_into_bool` and `eval_into_sym`
- [x] Add `eval_into_usize` so index code such as `vec-get` does not range-check an `i32` by hand
- [ ] Add a variadic arity helper (`vec-set!` now uses `get_exact_3_args`)
- [ ] Add a second registration form for the common case of evaluated arguments, e.g. `define_native_fn("hypot", |args: &[Expr], ctx| ...)`, keeping the unevaluated form for special-form-like natives
- [x] Re-export `eval_into_num`, `eval_into_str`, `eval_into_list`, `eval_into_usize`, `get_exact_3_args`, and `get_2_or_3_args` from the crate root

## P5 — `ForeignObject` trait (breaking, target 0.6.0)

Done. `Foreign` is `Rc<dyn ForeignObject>`. The trait's default `fmt` prints
`<type_name>`, and `trace` reports stored `Expr`s to the collector.
`register_foreign_tracer` is gone; `vec` prints `<vec 1 2 3>` and `dict` prints
its pairs. In-repo hosts, examples, and docs use the trait.

- [x] Introduce a `ForeignObject` trait with default methods: `type_name`, `fmt`, `trace(&self, &mut dyn FnMut(&Expr))`, `as_any`
- [x] Store `Rc<dyn ForeignObject>` so tracing lives on the type and `register_foreign_tracer` is no longer required for the common case
- [x] Use the trait's `fmt` so `write` and the REPL can print `<vec 1 2 3>`
- [x] Remove `register_foreign_tracer` and update `docs/tutorials/foreign.md`
- [x] Breaking change of its own; the `Proc::Native` signature change already landed with P1

## P6 — Extension and builder API

Mostly done. `Extension` has a blanket impl for `Fn(&Evaluator)`, so stateless
loaders such as `load_vec_procs` install as plain functions while host-bearing
ones are structs (`IoProcs`, `SysProcs`, `TurtleProcs`). `Evaluator::install`
returns `&Self` so calls chain. `TurtleProcs` runs its `repeat` macro through
`Evaluator::eval_prelude`, so errors inside `repeat` now point at the script
form. The builder is left out: both hosts read `Option` settings from the
command line and the existing setters handle that directly, so a builder would
only add a second way to spell `set_max_call_depth` and `set_gc_threshold`.

- [x] Add an `Extension` trait with `fn install(&self, evaluator: &Evaluator)` and `Evaluator::install`
- [x] Add `Evaluator::eval_prelude(src)` that strips spans the way the core prelude does
- [ ] Add `Evaluator::builder()` collecting `max_call_depth`, `gc_threshold`, prelude on/off, and extensions
- [x] Convert the `rusche-cli` and `rusche-turtle` loaders to extensions

## P7 — Named sources in spans

Done. A [`Span`](../src/span.rs) carries an interned source name (a `Loc` stays
line and column only, so `Expr` does not grow). `Evaluator::eval_str_named`
and `eval_source_named` tag a buffer; `eval_str` stays unnamed. `load`
evaluates with the file path as the source name and returns the inner error.
The CLI keeps the loaded text and prints that file when the span's name is
not the caller's buffer, so the old print-then-rethrow path is gone.

- [x] Add a source name to `Span` or to `Frame.call_site`
- [x] Add `eval_source_named` (or a source-name parameter on `eval_str`) so multi-file traces name each file
- [x] Simplify `rusche-cli`'s `load` error path once traces carry the file name
