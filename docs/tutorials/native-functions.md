# Tutorial: Writing a native function

Native functions are Rust procedures registered in a Rusche environment. They are
how a **host** adds I/O, domain APIs, and other capabilities the core language
does not provide — the same mechanism [`rusche-cli`](../rusche-cli.md) uses for
`display` and friends.

A runnable version of the worked example lives in
[`examples/tutorial-native`](../../examples/tutorial-native/main.rs).

## Signature

A native procedure is a function pointer matching
[`NativeFunc`](https://docs.rs/rusche/latest/rusche/type.NativeFunc.html):

```rust
use rusche::{EvalContext, EvalResult, List};

pub type NativeFunc = fn(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult;
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
`Expr::Proc(Rc::new(Proc::Native { name, func }), None)`.

## Evaluate arguments

Arguments arrive **unevaluated**. Evaluate them yourself with `eval`, or with the
helpers in [`rusche::utils`](https://docs.rs/rusche/latest/rusche/utils/index.html):

| Helper | Result |
| --- | --- |
| `eval(expr, context)` | Any `Expr` |
| `eval_into_num` | `f64` |
| `eval_into_int` | integer `i32` (no fractional part) |
| `eval_into_str` | `String` |
| `eval_into_foreign` | `Rc<dyn Any>` for a `Foreign` object |

Fixed arity:

| Helper | Expects |
| --- | --- |
| `get_exact_1_arg` | exactly one argument |
| `get_exact_2_args` | exactly two |
| `get_exact_3_args` | exactly three |

Variable arity: iterate `args` and evaluate each element, as
[`display`](../../crates/rusche-cli/src/builtin/io.rs) does in `rusche-cli`.

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
    utils::{eval_into_num, get_exact_3_args},
    EvalContext, EvalResult, Evaluator, Expr, List,
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
cargo run --example tutorial-native
```

## Limits of `NativeFunc`

`NativeFunc` is a plain function pointer: it cannot capture host state in a
closure. Shared or mutable host data belongs in a [`Foreign`](foreign.md) object
that native procedures create and update.

A native returns a finished value, so anything it evaluates with `eval` is a
nested call: it counts towards the call depth and is never a tail call. If you
need a control-flow form whose body stays in tail position (a `when`, a custom
loop, …), write it as a `defmacro` in Rusche that expands to `if`/`begin`, and
load it the way the prelude does, instead of implementing it as a native.

## Next steps

- [How to write a foreign object wrapper](foreign.md)
- Simpler host embedding: [How to embed the Rusche interpreter](embedding.md)
- Production-style natives: [`rusche-cli` builtins](../../crates/rusche-cli/src/builtin/)
