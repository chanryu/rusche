# `rusche-cli`

[`rusche-cli`](../examples/rusche-cli) is an **example host application**, not part of
the core `rusche` crate. It shows how to build a standalone interpreter with a REPL
and file runner on top of the library.

The [language reference](language-reference.md) documents only the core language
(built-ins and prelude). Everything below is added by `rusche-cli` when it starts.

## What it loads

On startup, `rusche-cli` creates `Evaluator::with_prelude()` and then registers:

| Layer | Source | Contents |
| --- | --- | --- |
| Core built-ins + prelude | `rusche` crate | See [language reference](language-reference.md) |
| I/O procedures | [`builtin/io.rs`](../examples/rusche-cli/builtin/io.rs) | `display`, `newline`, `read` |
| Vector foreign type | [`builtin/vec.rs`](../examples/rusche-cli/builtin/vec.rs) | `vec?`, `vec-make`, `vec-push`, `vec-pop`, `vec-get` |
| Scheme-style aliases | [`builtin/scheme.rs`](../examples/rusche-cli/builtin/scheme.rs) | `number?`, `modulo`, `string-append`, … |

Scripts under [`examples/*.rsc`](../examples) are written for this host: they use
`display` / `read` / `newline` and the Scheme aliases (`modulo`, `string->number`,
…), not only core names.

## I/O

| Procedure | Behaviour |
| --- | --- |
| `display` | Prints each argument; strings are printed without surrounding quotes |
| `newline` | Prints a newline |
| `read` | Reads one line from stdin and returns it as a string (trailing newline trimmed) |

These are ordinary native procedures. A different host can omit them or replace them.

## Vectors

`rusche-cli` wraps `RefCell<Vec<Expr>>` as a [`Foreign`](tutorials/foreign.md) object:

```scheme
(define v (vec-make))
(vec-push v 1)
(vec-push v 2)
(vec-get v 0)    ; 1
(vec-pop v)      ; 2
(vec? v)         ; 1
```

A GC tracer is registered so closures stored in a vector stay reachable. See the
[foreign object tutorial](tutorials/foreign.md) for the general pattern.

## Scheme-style aliases

Core procedures keep short names (`num?`, `%`, `str-append`, `num-parse`, …).
`rusche-cli` binds familiar Scheme spellings as aliases in the root environment:

| Alias | Core name |
| --- | --- |
| `number?` | `num?` |
| `modulo` | `%` |
| `string->number` | `num-parse` |
| `string?` | `str?` |
| `string-append` | `str-append` |
| `string-compare` | `str-compare` |
| `string-length` | `str-length` |
| `substring` | `str-slice` |
| `pair?` | `(lambda (x) (not (atom? x)))` |

An embedding that wants Scheme names can copy
[`scheme.rs`](../examples/rusche-cli/builtin/scheme.rs); the core library does not
define them.

## Running

```bash
cargo run --example rusche-cli
cargo run --example rusche-cli -- examples/fizzbuzz.rsc
```

## Related docs

- [Language reference](language-reference.md) — core language only
- [Embedding tutorial](tutorials/embedding.md) — using the crate without `rusche-cli`
- [Native functions](tutorials/native-functions.md) / [Foreign wrappers](tutorials/foreign.md) — how hosts add batteries like I/O and `vec`
