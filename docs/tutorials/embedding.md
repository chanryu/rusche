# Tutorial: Embedding the Rusche interpreter

This guide shows how to embed the **core** `rusche` crate in a Rust application:
create an evaluator, parse source, evaluate expressions, and exchange values with
the host. It does not use [`rusche-cli`](../rusche-cli.md); that example host's
extras (I/O, `vec`, `dict`) are optional batteries you can add yourself.

A runnable version of the snippets lives in
[`examples/tutorial-embed`](../../examples/tutorial-embed/main.rs).

## Choose an evaluator

Rusche exposes three constructors on [`Evaluator`](https://docs.rs/rusche/latest/rusche/struct.Evaluator.html):

| Constructor | Contents |
| --- | --- |
| `Evaluator::new()` | Empty root environment — only the evaluator forms (`quote`, `quasiquote`, `begin`, `if`, `eval`, `apply`) |
| `Evaluator::with_builtin()` | Core native procedures (`num-add`, `car`, `lambda`, …) |
| `Evaluator::with_prelude()` / `Evaluator::default()` | Built-ins plus the prelude (`+`, `sqrt`, `let`, `while`, `and`, `or`, …) |

For scripting, start with `with_prelude()` (or `Default`). Use `new()` or
`with_builtin()` when you want a smaller surface.

```rust
use rusche::Evaluator;

let evaluator = Evaluator::with_prelude();
// same as Evaluator::default()
```

## Evaluate source with `eval_str`

The usual host path is `Evaluator::eval_str`, which tokenizes, parses every
top-level form, evaluates each one, and returns the last value (or `()` if the
source is empty). Lex, parse, and eval failures share the unified
[`Error`](https://docs.rs/rusche/latest/rusche/enum.Error.html) type:

```rust
use rusche::{Evaluator, Expr};

let evaluator = Evaluator::default();
let result = evaluator.eval_str("(+ 1 (% 9 2))").unwrap();
assert_eq!(result, Expr::from(2));
```

A file or buffer can hold several top-level forms:

```rust
use rusche::Evaluator;

fn eval_script(evaluator: &Evaluator, source: &str) -> Result<(), rusche::Error> {
    evaluator.eval_str(source)?;
    Ok(())
}
```

`Error` implements `Display` and `std::error::Error`. Use `message()` for the
bare text (without a leading span), `span()` for the source location, `help()`
for an optional hint, and `trace()` for the call stack collected while
unwinding. The [`rusche-cli`](../../crates/rusche-cli/src/main.rs) host prints
those against the original source lines.

### Lower-level pipeline

When you need incremental parsing (for example a multi-line REPL), use the
pieces underneath `eval_str`:

```rust
use rusche::{tokenize, Evaluator, Expr, Parser};

let source = "(+ 1 (% 9 2))";

let tokens = tokenize(source, None).unwrap();
let mut parser = Parser::with_tokens(tokens);
let expr = parser.parse().unwrap().unwrap();

let evaluator = Evaluator::default();
let result = evaluator.eval(&expr).unwrap();

assert_eq!(result, Expr::from(2));
```

- `tokenize(text, loc)` turns source into tokens. Pass `Some(Loc::…)` if you want
  spans relative to a known origin; `None` starts at the default location.
- `Parser::parse` returns `Ok(None)` when there are no more complete expressions,
  `Ok(Some(expr))` for one expression, or a `ParseError`.
- `Evaluator::eval` runs the expression in the root context and may trigger
  automatic garbage collection afterward.

## Exchange values with the host

### Define host values into the environment

Use `root_env().define` to inject numbers, strings, or other `Expr` values before
running script code:

```rust
use rusche::{Evaluator, Expr};

let evaluator = Evaluator::default();
evaluator.root_env().define("greeting", Expr::from("hello"));

let result = evaluator
    .eval_str("(str-append greeting \" world\")")
    .unwrap();

assert_eq!(result, Expr::from("hello world"));
```

For native procedures and foreign objects, see
[How to write a native function](native-functions.md) and
[How to write a foreign object wrapper](foreign.md).

### Read results back

Use the accessors. They ignore the span stored on each value:

```rust
use rusche::Expr;

fn describe(expr: &Expr) -> String {
    if let Some(n) = expr.as_num() {
        return format!("number {n}");
    }
    if let Some(b) = expr.as_bool() {
        return format!("boolean {b}");
    }
    if let Some(s) = expr.as_str() {
        return format!("string {s:?}");
    }
    if let Some(name) = expr.as_sym() {
        return format!("symbol {name}");
    }
    if let Some(list) = expr.as_list() {
        return format!("list {list}");
    }
    if let Some(proc) = expr.as_proc() {
        return format!("procedure <{}>", proc.fingerprint());
    }
    format!("foreign {expr}")
}
```

`as_foreign::<T>()` downcasts a foreign object to `Rc<T>`. Build one with
`Expr::foreign(value)`.

Numbers are `f64`. Booleans are `true` / `false`; conditions in `if` must be
booleans. See the [language reference](../language-reference.md) for the full
type system.

## Call a procedure from Rust

`Evaluator::call` invokes a procedure with values the host already has. The
arguments are not evaluated again, so a list stays data:

```rust
use rusche::{Evaluator, Expr};

let evaluator = Evaluator::default();
evaluator.eval_str("(define (double x) (* x 2))").unwrap();
let double = evaluator.eval_str("double").unwrap();
assert_eq!(
    evaluator.call(&double, &[Expr::from(21)]).unwrap(),
    Expr::from(42)
);
```

Inside a native, use `EvalContext::call` the same way. `eval_into_proc` checks
that an argument evaluated to a procedure:

```rust
use rusche::{
    eval, eval_into_proc, get_exact_3_args, EvalContext, EvalResult, List,
};

fn apply2(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (proc_expr, a_expr, b_expr) = get_exact_3_args(proc_name, args)?;
    let proc = eval_into_proc(proc_name, proc_expr, context)?;
    let a = eval(a_expr, context)?;
    let b = eval(b_expr, context)?;
    context.call(&proc, &[a, b])
}
```

`(apply2 (lambda (a b) (+ a b)) 20 22)` is `42`. Macros passed to `call` receive
the arguments as forms, matching `(apply macro args)`.

## Interrupting evaluation

`set_interrupt(Some(flag))` watches an `Arc<AtomicBool>`. The eval loop loads it
on the first step and every `INTERRUPT_CHECK_INTERVAL` steps after that (64).
When the flag is set, evaluation fails with `ErrorKind::Interrupted` and the
message `evaluation interrupted`. The evaluator does not clear the flag. Pass
`None` to stop watching.

`set_step_budget(Some(n))` fails the same way, with message
`step budget (n) exceeded`, once a top-level `eval`, `eval_str`, or `call` has
taken more than `n` steps. Each of those calls starts the count over when
evaluation is not already inside a procedure, so a native that re-enters `eval`
or `call` keeps the budget of the script that called it. Pass `None` to disable
the cap. One step is one trip around the eval loop; atoms that never enter the
loop do not count.

## Call depth and garbage collection

- **Call depth.** Non-tail calls are capped by `Evaluator::max_call_depth()`
  (default `DEFAULT_MAX_CALL_DEPTH`). Exceeding it is an error instead of a Rust
  stack overflow. Adjust with `set_max_call_depth` for the thread you evaluate on.
  Tail calls do not count (see [Tail Calls](../language-reference.md#tail-calls)).
  A native procedure counts as one level while it runs, and anything it
  evaluates with `eval` or `call` nests inside it.
- **Garbage collection.** After a top-level `eval`, the evaluator may collect
  unreachable environments once their count reaches `gc_threshold()` (default
  `DEFAULT_GC_THRESHOLD`; pass `None` to `set_gc_threshold` to disable). Call
  `collect_garbage` anytime for an explicit collection.

Reachability starts from the root environment and the value `eval` just returned.
If the host stores `Expr` values (especially closures) inside a `Foreign` object,
register a tracer so the collector can see them — covered in
[How to write a foreign object wrapper](foreign.md).

## Next steps

- [How to write a native function](native-functions.md)
- [How to write a foreign object wrapper](foreign.md)
- Standalone REPL and file runner: [`crates/rusche-cli`](../../crates/rusche-cli/)
- Language surface: [language reference](../language-reference.md)
