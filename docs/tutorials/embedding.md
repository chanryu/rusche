# Tutorial: Embedding the Rusche interpreter

This guide shows how to embed the **core** `rusche` crate in a Rust application:
create an evaluator, parse source, evaluate expressions, and exchange values with
the host. It does not use [`rusche-cli`](../rusche-cli.md); that example host's
extras (I/O, `vec`, Scheme aliases) are optional batteries you can add yourself.

A runnable version of the snippets lives in
[`examples/tutorial-embed`](../../examples/tutorial-embed/main.rs).

## Choose an evaluator

Rusche exposes three constructors on [`Evaluator`](https://docs.rs/rusche/latest/rusche/struct.Evaluator.html):

| Constructor | Contents |
| --- | --- |
| `Evaluator::new()` | Empty root environment — no primitives |
| `Evaluator::with_builtin()` | Core native procedures (`num-add`, `car`, `lambda`, …) |
| `Evaluator::with_prelude()` / `Evaluator::default()` | Built-ins plus the prelude (`+`, `%`, `let`, `while`, `#t`, `#f`, …) |

For scripting, start with `with_prelude()` (or `Default`). Use `new()` or
`with_builtin()` when you want a smaller surface.

```rust
use rusche::Evaluator;

let evaluator = Evaluator::with_prelude();
// same as Evaluator::default()
```

## Evaluate one expression

The usual pipeline is tokenize → parse → evaluate:

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

## Evaluate a multi-expression script

A file or buffer can hold several top-level forms. Keep calling `parse` until it
returns `Ok(None)`, and handle lex, parse, and eval errors separately:

```rust
use rusche::{tokenize, Evaluator, LexError, ParseError, Parser};

fn eval_script(evaluator: &Evaluator, source: &str) -> Result<(), String> {
    let tokens = tokenize(source, None).map_err(|e| match e {
        LexError::InvalidNumber(span) => format!("invalid number at {span}"),
        LexError::IncompleteString(span) => format!("incomplete string at {span}"),
    })?;

    let mut parser = Parser::with_tokens(tokens);
    loop {
        match parser.parse() {
            Ok(None) => return Ok(()),
            Ok(Some(expr)) => {
                evaluator.eval(&expr).map_err(|e| e.message)?;
            }
            Err(ParseError::IncompleteExpr(token)) => {
                return Err(format!("incomplete expression starting at {}", token.span()));
            }
            Err(ParseError::UnexpectedToken(token)) => {
                return Err(format!("unexpected token at {}", token.span()));
            }
        }
    }
}
```

`EvalError` carries a `message` and an optional `Span`. The
[`rusche-cli`](../../examples/rusche-cli/main.rs) example prints those spans
against the original source lines.

## Exchange values with the host

### Define host values into the environment

Use `root_env().define` to inject numbers, strings, or other `Expr` values before
running script code:

```rust
use rusche::{tokenize, Evaluator, Expr, Parser};

let evaluator = Evaluator::default();
evaluator.root_env().define("greeting", Expr::from("hello"));

let tokens = tokenize("(str-append greeting \" world\")", None).unwrap();
let expr = Parser::with_tokens(tokens).parse().unwrap().unwrap();
let result = evaluator.eval(&expr).unwrap();

assert_eq!(result, Expr::from("hello world"));
```

For native procedures and foreign objects, see
[How to write a native function](native-functions.md) and
[How to write a foreign object wrapper](foreign.md).

### Read results back

Match on `Expr` variants after `eval`:

```rust
use rusche::Expr;

fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Num(n, _) => format!("number {n}"),
        Expr::Str(s, _) => format!("string {s:?}"),
        Expr::Sym(name, _) => format!("symbol {name}"),
        Expr::List(list, _) => format!("list {list}"),
        Expr::Proc(proc, _) => format!("procedure <{}>", proc.fingerprint()),
        Expr::Foreign(object) => format!("foreign {:p}", object),
        Expr::TailCall { .. } => unreachable!("TailCall is internal"),
    }
}
```

Numbers are `f64`. The empty list `()` is the only false value; everything else
is truthy. See the [language reference](../language-reference.md) for the full
type system.

## Call depth and garbage collection

- **Call depth.** Non-tail calls are capped by `Evaluator::max_call_depth()`
  (default `DEFAULT_MAX_CALL_DEPTH`). Exceeding it is an error instead of a Rust
  stack overflow. Adjust with `set_max_call_depth` for the thread you evaluate on.
  Tail calls do not count.
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
- Standalone REPL and file runner: [`examples/rusche-cli`](../../examples/rusche-cli/)
- Language surface: [language reference](../language-reference.md)
