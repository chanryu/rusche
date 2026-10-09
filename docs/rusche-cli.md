# `rusche-cli`

[`rusche-cli`](../crates/rusche-cli) is an **example host application**, not part of
the core `rusche` crate. It shows how to build a standalone interpreter with a REPL
and file runner on top of the library.

The [language reference](language-reference.md) documents only the core language
(built-ins and prelude). Everything below is added by `rusche-cli` when it starts.

## Installation

```bash
cargo install --path crates/rusche-cli
# or run from the workspace:
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
```

## What it loads

On startup, `rusche-cli` creates `Evaluator::with_prelude()` (unless `--no-prelude`)
and then registers:

| Layer | Source | Contents |
| --- | --- | --- |
| Core built-ins + prelude | `rusche` crate | See [language reference](language-reference.md) |
| I/O procedures | [`builtin/io.rs`](../crates/rusche-cli/src/builtin/io.rs) | `display`, `write`, `newline`, `read`, `exit`, `load` |
| System procedures | [`builtin/sys.rs`](../crates/rusche-cli/src/builtin/sys.rs) | `getenv`, `clock`, `random`, `command-line` |
| Vector foreign type | [`builtin/vec.rs`](../crates/rusche-cli/src/builtin/vec.rs) | `vec?`, `vec-make`, `vec`, `vec-push`, `vec-pop`, `vec-get`, `vec-set!`, `vec-length`, `vec->list`, `list->vec` |
| Scheme-style aliases | [`builtin/scheme.rs`](../crates/rusche-cli/src/builtin/scheme.rs) | `number?`, `modulo`, `string-append`, `vector?`, … |

Scripts under [`examples/*.rsc`](../examples) are written for this host: they use
`display` / `read` / `newline` and the Scheme aliases (`modulo`, `string->number`,
…), not only core names.

## Command-line usage

```text
Usage: rusche-cli [OPTIONS] [FILE [ARGS...]]
       rusche-cli [OPTIONS] -e EXPR
       rusche-cli [OPTIONS] -
```

| Option | Meaning |
| --- | --- |
| `-h`, `--help` | Show usage |
| `-V`, `--version` | Show version |
| `-e`, `--eval EXPR` | Evaluate `EXPR` and exit |
| `--max-call-depth N` | Set maximum call depth |
| `--gc-threshold N\|off` | Set GC threshold, or disable automatic GC |
| `--no-prelude` | Start with built-ins only (no Scheme aliases) |
| `--no-color` | Disable colored diagnostics |

With no `FILE` and a TTY on stdin, starts a REPL. With no `FILE` and a pipe on
stdin, reads the script from stdin. Use `-` to force reading from stdin.

A leading `#!` line in a script file is skipped (shebang support).

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Lex, parse, or evaluation error in the script/`-e` expression |
| `2` | Usage error or I/O failure (unknown flag, unreadable file, …) |

`(exit n)` terminates with code `n` (0–255). Diagnostics are written to **stderr**.

## REPL

```bash
cargo run -p rusche-cli
```

- Multi-line input is validated before submission (incomplete forms keep the
  editor open). Bracket matching and symbol completion are enabled.
- History is stored in `~/.rusche_history`.
- Ctrl+C discards the current input; Ctrl+D exits.
- `()` results are not printed; a newline is inserted before a result if the
  last host output did not end with one.

### Meta-commands

| Command | Action |
| --- | --- |
| `,help` | Show meta-command help |
| `,load <file>` | Evaluate a file in the current environment |
| `,env` | List root environment bindings |
| `,gc` | Run garbage collection and report unreachable envs |
| `,quit` | Exit the REPL |

## I/O

| Procedure | Behaviour |
| --- | --- |
| `display` | Prints each argument; strings are printed without surrounding quotes |
| `write` | Prints each argument with `Display` (strings quoted) |
| `newline` | Prints a newline (no arguments) |
| `read` | Reads one line from stdin and returns it as a string (trailing newline trimmed). At EOF returns `()` |
| `exit` | Flushes stdout and exits with optional integer code (default 0) |
| `load` | Reads a file and evaluates it in the current environment |

These are ordinary native procedures. A different host can omit them or replace them.

## System

| Procedure | Behaviour |
| --- | --- |
| `getenv` | Returns the environment variable value as a string, or `()` if unset |
| `clock` | Seconds since the UNIX epoch as a number |
| `random` | Uniform random number in `[0, 1)` |
| `command-line` | List of strings: program path, then script path (if any), then script args |

## Vectors

`rusche-cli` wraps `RefCell<Vec<Expr>>` as a [`Foreign`](tutorials/foreign.md) object:

```scheme
(define v (vec-make))
(vec-push v 1)
(vec-push v 2)
(vec-get v 0)    ; 1
(vec-pop v)      ; 2
(vec? v)         ; 1

(define w (vec 10 20 30))
(vec-length w)   ; 3
(vec-set! w 1 99)
(vec->list w)    ; (10 99 30)
(list->vec '(a b c))
```

A GC tracer is registered so closures stored in a vector stay reachable. See the
[foreign object tutorial](tutorials/foreign.md) for the general pattern.

## Scheme-style aliases

Core procedures keep short names (`num?`, `%`, `str-append`, `str->num`, …).
`rusche-cli` binds familiar Scheme spellings as aliases in the root environment
(skipped with `--no-prelude`):

| Alias | Core / host name |
| --- | --- |
| `number?` | `num?` |
| `modulo` | `%` |
| `string->number` | `str->num` |
| `number->string` | `num->str` |
| `string?` | `str?` |
| `string-append` | `str-append` |
| `string-compare` | `str-compare` |
| `string-length` | `str-length` |
| `substring` | `str-slice` |
| `string=?` | via `string-compare` |
| `symbol?` | `sym?` |
| `procedure?` | `proc?` |
| `symbol->string` | `sym->str` |
| `string->symbol` | `str->sym` |
| `pair?` | `(lambda (x) (not (atom? x)))` |
| `vector?` | `vec?` |
| `make-vector` | `vec-make` |
| `vector-length` | `vec-length` |
| `vector-ref` | `vec-get` |
| `vector-set!` | `vec-set!` |
| `list-ref` | indexed `car`/`cdr` |
| `assq` | `assoc` |

An embedding that wants Scheme names can copy
[`scheme.rs`](../crates/rusche-cli/src/builtin/scheme.rs); the core library does not
define them.

## Running examples

```bash
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
cargo run -p rusche-cli -- -e '(+ 1 2)'

# After cargo install --path crates/rusche-cli:
chmod +x examples/fizzbuzz.rsc
./examples/fizzbuzz.rsc
```

## Related docs

- [Language reference](language-reference.md) — core language only
- [Embedding tutorial](tutorials/embedding.md) — using the crate without `rusche-cli`
- [Native functions](tutorials/native-functions.md) / [Foreign wrappers](tutorials/foreign.md) — how hosts add batteries like I/O and `vec`
