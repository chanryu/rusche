# Tutorial: Writing a native function

Native functions are Rust procedures registered in a Rusche environment. They are
how a **host** adds I/O, domain APIs, and other capabilities the core language
does not provide — the same mechanism [`rusche-cli`](../rusche-cli.md) uses for
`write` and friends.

Runnable versions of the examples live in
[`examples/tutorial-native`](../../examples/tutorial-native): `clamp.rs` is the
worked example below, and `capture.rs` shows natives that capture host state.

## Signature

A native procedure is any `'static` function of the shape
`Fn(&str, &List, &EvalContext) -> EvalResult`: a plain function item, or a
closure that captures host state. `define_native_proc` wraps it in a
[`NativeFunc`](https://docs.rs/rusche/latest/rusche/struct.NativeFunc.html).

```rust
fn clamp(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult { /* ... */ }
```

| Parameter | Role |
| --- | --- |
| `proc_name` | The name used at registration — include it in error messages |
| `args` | Unevaluated argument expressions (as a list) |
| `context` | Current environment and evaluation state |

Register with `Env::define_native_proc`:

```rust
use rusche::Evaluator;

let evaluator = Evaluator::with_prelude();
evaluator.root_env().define_native_proc("clamp", clamp);
```

`define_native_proc` is shorthand for binding
`Expr::Proc(Rc::new(Proc::Native { name: Rc::from(name), func: NativeFunc::new(func) }), None)`.
Two natives compare equal only when they share that `NativeFunc` allocation, so a
closure that captures different state is a different procedure.

## Evaluate arguments

Arguments arrive **unevaluated**. Evaluate them yourself with `eval`, or with the
helpers in [`rusche::utils`](https://docs.rs/rusche/latest/rusche/utils/index.html):

| Helper | Result |
| --- | --- |
| `eval(expr, context)` | Any `Expr` |
| `eval_into_num` | `f64` |
| `eval_into_int` | integer `i32` (no fractional part) |
| `eval_into_str` | `String` |
| `eval_into_foreign` | `Rc<dyn ForeignObject>` |

Fixed arity:

| Helper | Expects |
| --- | --- |
| `get_exact_1_arg` | exactly one argument |
| `get_exact_2_args` | exactly two |
| `get_exact_3_args` | exactly three |

Variable arity: iterate `args` and evaluate each element, as
[`write`](../../crates/rusche-cli/src/builtin/io.rs) does in `rusche-cli`.

## Return values and errors

Return any `Expr`. Convenience conversions include:

- numbers: `Expr::from(3.0)` / `Expr::from(3)`
- strings: `Expr::from("hello")`
- booleans: Rust `true` / `false` become `Expr::Bool` (printed as `true` / `false`)
- "nothing found": return `false` (so callers can use `or-else` or compare with `eq?`)
- side effects with no useful value: `NIL` (the empty list)

On failure, return an [`EvalError`](https://docs.rs/rusche/latest/rusche/struct.EvalError.html)
built with `EvalError::new(ErrorKind::…, message).with_span(…)`. Prefer the
offending argument's `span()` so host error printers can underline the right
token. Use `ErrorKind` values such as `Arity`, `Type`, `InvalidForm`, or `Other`
so hosts and tests can match without parsing message text.

## Worked example: `clamp`

```rust
use rusche::{
    eval_into_num, get_exact_3_args, EvalContext, EvalResult, Evaluator, Expr, List,
};

fn clamp(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (value_expr, lo_expr, hi_expr) = get_exact_3_args(proc_name, args)?;
    let value = eval_into_num(proc_name, value_expr, context)?;
    let lo = eval_into_num(proc_name, lo_expr, context)?;
    let hi = eval_into_num(proc_name, hi_expr, context)?;

    Ok(Expr::from(value.clamp(lo, hi)))
}

fn main() {
    let evaluator = Evaluator::with_prelude();
    evaluator.root_env().define_native_proc("clamp", clamp);

    let result = evaluator
        .eval(
            &rusche::Parser::with_tokens(
                rusche::tokenize("(clamp 15 0 10)", None).unwrap(),
            )
            .parse()
            .unwrap()
            .unwrap(),
        )
        .unwrap();

    assert_eq!(result, Expr::from(10));
}
```

Run it with:

```bash
cargo run --example tutorial-native-clamp
```

## Capturing host state

The closure passed to `define_native_proc` can capture host data. The capture
must be `'static`, so share it with `Rc` (and `Cell` or `RefCell` when it
changes) rather than borrowing a local:

```rust
use std::cell::Cell;
use std::rc::Rc;

let count = Rc::new(Cell::new(0));
let count_for_proc = Rc::clone(&count);
evaluator.root_env().define_native_proc("bump", move |proc_name, args, _| {
    if !args.is_nil() {
        return Err(rusche::arity_error(proc_name, 0..=0, args.len()));
    }
    let next = count_for_proc.get() + 1;
    count_for_proc.set(next);
    Ok(rusche::Expr::from(next))
});
```

`(bump)` then returns `1`, then `2`, and `count` on the host reads the same
numbers. This is how [`rusche-cli`](../../crates/rusche-cli/src/host.rs) shares
its command line and [`rusche-turtle-rs`](../../crates/rusche-turtle-rs/src/builtin.rs)
shares the turtle.

A single counter could still have lived in a global. What a function pointer
could not do is give each evaluator its own host.
[`capture.rs`](../../examples/tutorial-native/capture.rs) registers `deposit`
and `balance` on two evaluators, each closure capturing a different `Bank`, and
shows that one script cannot see the other's balance. A thread-local host would
have mixed them.

```bash
cargo run --example tutorial-native-capture
```

Use a [`Foreign`](foreign.md) object when the value itself should be a Rusche
value that scripts pass around. Use a capture when the state belongs to the
host and scripts only call into it.

A native returns a finished value, so anything it evaluates with `eval` is a
nested call: it counts towards the call depth and is never a tail call. If you
need a control-flow form whose body stays in tail position (a `when`, a custom
loop, …), write it as a `defmacro` in Rusche that expands to `if`/`begin` and
load it with `Evaluator::eval_prelude` instead of implementing it as a native:

```rust
evaluator
    .eval_prelude("(defmacro (unless c *body) `(if ,c () (begin ,@body)))")
    .unwrap();
```

`eval_prelude` strips source spans before evaluating, the same way the core
prelude is loaded, so an error raised later inside `unless` points at the
script form that used it rather than at a line inside your prelude string.

## Bundling natives as an extension

A host usually registers a group of related natives together. Any
`Fn(&Evaluator)` is an [`Extension`](https://docs.rs/rusche/latest/rusche/trait.Extension.html),
so a loader function works with `Evaluator::install` as is, and `install`
returns the evaluator so calls chain. Implement the trait on a struct when the
group captures host state:

```rust
use std::rc::Rc;
use rusche::{Evaluator, Expr, Extension};

fn math_procs(evaluator: &Evaluator) {
    evaluator.root_env().define_native_proc("clamp", clamp);
}

struct Greeter(Rc<String>);

impl Extension for Greeter {
    fn install(&self, evaluator: &Evaluator) {
        let name = Rc::clone(&self.0);
        evaluator
            .root_env()
            .define_native_proc("greet", move |_, _, _| Ok(Expr::from(format!("hi {name}"))));
    }
}

let evaluator = Evaluator::with_prelude();
evaluator
    .install(math_procs)
    .install(Greeter(Rc::new("rusche".into())));
```

[`rusche-cli`](../../crates/rusche-cli/src/main.rs) installs `IoProcs` and
`SysProcs` (which hold the host) alongside the plain `load_vec_procs` and
`load_dict_procs` functions, and
[`rusche-turtle-rs`](../../crates/rusche-turtle-rs/src/builtin.rs) installs
`TurtleProcs`, whose `install` registers the natives and then runs the turtle
prelude through `eval_prelude`.

## Next steps

- [How to write a foreign object wrapper](foreign.md)
- Simpler host embedding: [How to embed the Rusche interpreter](embedding.md)
- Production-style natives: [`rusche-cli` builtins](../../crates/rusche-cli/src/builtin/)
