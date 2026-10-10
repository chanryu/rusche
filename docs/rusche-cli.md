# `rusche-cli`

[`rusche-cli`](../crates/rusche-cli) is an example host, not part of the core
`rusche` crate: a REPL and file runner, plus I/O, `vec`, and `dict`.
The [language reference](language-reference.md) documents only the core language.

## Installation

```bash
cargo install --path crates/rusche-cli
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
```

`cargo install` puts `rusche-cli` on `PATH`, which `#!/usr/bin/env rusche-cli` needs.

## What it loads

| Layer | Source | Contents |
| --- | --- | --- |
| Core built-ins + prelude | `rusche` crate | [Language reference](language-reference.md). Built-ins always; prelude omitted with `--no-prelude`. |
| I/O | [`builtin/io.rs`](../crates/rusche-cli/src/builtin/io.rs) | `display`, `write`, `newline`, `read`, `exit`, `load` |
| System | [`builtin/sys.rs`](../crates/rusche-cli/src/builtin/sys.rs) | `getenv`, `clock`, `random`, `command-line` |
| Vectors | [`builtin/vec.rs`](../crates/rusche-cli/src/builtin/vec.rs) | `vec?`, `vec`, `vec-push`, `vec-pop`, `vec-get`, `vec-set!`, `vec-length`, `vec->list`, `list->vec` |
| Dicts | [`builtin/dict.rs`](../crates/rusche-cli/src/builtin/dict.rs) | `dict?`, `dict`, `dict-get`, `dict-set!`, `dict-has?`, `dict-remove!`, `dict-length`, `dict-keys`, `dict->list`, `list->dict` |

I/O, system, vector, and dict procedures are always registered. Scripts under
[`examples/*.rsc`](../examples) are written for this host.

## Running

```text
Usage: rusche-cli [OPTIONS] [FILE [ARGS...]]
       rusche-cli [OPTIONS] -e EXPR
       rusche-cli [OPTIONS] -
```

Options may appear in any order. A second `-e` replaces the first. The first
file path or `-` ends option parsing: that is the script, later arguments are
script arguments (even when they look like options), and an earlier `-e` is
dropped. `--` and `--flag=value` are unknown options; a value is always the
next argument.

| Invocation | What runs | Values printed | `(command-line)` |
| --- | --- | --- | --- |
| `rusche-cli` on a terminal | REPL | Each value other than `()` | `(program)` |
| `rusche-cli` with stdin piped | The pipe, as a script | None | `(program)` |
| `rusche-cli FILE ARGS...` | The file | None | `(program FILE ARGS...)` |
| `rusche-cli - ARGS...` | Stdin | None | `(program "-" ARGS...)` |
| `rusche-cli -e EXPR` | `EXPR` (a later file or `-` replaces it) | The last value, unless it is `()` | `(program)` |

`program` is `argv[0]`. Every top-level form runs. An empty script prints
nothing. File, pipe, and `-` produce output only through `display`, `write`,
and `newline`.

```bash
rusche-cli --no-color examples/counter.rsc
rusche-cli examples/mandelbrot.rsc 80 30    # 80 and 30 are script args
rusche-cli -e '(+ 1 2)'
echo '(+ 1 2)' | rusche-cli
```

Unknown options, missing values, and a non-integer `N` print `error:` plus the
usage text on stderr and exit 2, even when `--help` is also present. After a
valid parse, `--help` wins over `--version`, and both exit before any script.

A leading `#!` line is skipped for a command-line script and for stdin.
`(load)` and `,load` leave the line in the file. A file that is only a shebang
is an empty program.

### Options

| Option | Meaning |
| --- | --- |
| `-h`, `--help` | Print usage on stdout and exit 0. |
| `-V`, `--version` | Print `rusche-cli` and the package version on stdout and exit 0. |
| `-e`, `--eval EXPR` | Evaluate `EXPR` and exit. |
| `--max-call-depth N` | Maximum call depth (default 1000). Tail calls do not count. See [Tail calls](language-reference.md#tail-calls). |
| `--gc-threshold N` | Collect automatically once live environments reach `N` (default 10000). |
| `--gc-threshold off` | Leave automatic collection off. `,gc` in the REPL still collects. |
| `--no-prelude` | Built-ins plus I/O, system, `vec`, and `dict`. No prelude (`cond`, `let`, `+`, …). |
| `--no-color` | Turn off color in the banner, REPL values, meta-command help, and diagnostics. |

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. The REPL exits 0 on Ctrl+D or `,quit`, including after failed entries. |
| `1` | Lex, parse, or evaluation error in a script, stdin, or `-e`. |
| `2` | Usage error, or the script file or stdin could not be read. |
| `(exit n)` | The process ends at once with status `n`. See [`exit`](#exit). |

Diagnostics go to stderr. Script output goes to stdout.

## REPL

A terminal with no file, no `-`, and no `-e` starts the REPL. The prompt is
`repl❯ `.

- An unfinished form or string stays open for another line. One line may hold several forms. A blank line is ignored.
- Tab completes the identifier at the cursor from names in the root environment. Matching brackets are highlighted.
- History is `$HOME/.rusche_history` when `HOME` is set, loaded at startup and saved on Ctrl+D or `,quit`.
- Ctrl+C drops the current line. Ctrl+D exits.
- `()` is not printed. Any other result is printed in green. If `display` or `write` did not end in a newline, the next prompt stays on that line, and a newline is inserted before the next result.
- `(read)` takes the next typed line as data.

A line whose first non-whitespace character is `,` is a meta-command:

| Command | Action |
| --- | --- |
| `,help`, `,h` | Show this list. |
| `,load FILE` | Evaluate `FILE` in the current environment and print the last value when it is not `()`. |
| `,env` | Print root-environment names, sorted. |
| `,gc` | Collect and print `collected; unreachable before=B, after=A`. |
| `,quit`, `,q`, `,exit` | Leave the REPL. |

An unknown command prints `unknown meta-command: ,name (try ,help)`.

## Diagnostics

Errors go to stderr in a rustc-style layout: message, source header, excerpt with
carets, optional help, and a call trace (innermost first). Tail calls replace
the active frame, so the trace lists the non-tail call chain.

```text
error: `7` is not a symbol
  --> example.rsc:2:16
  1| (define plus
  2|     (lambda (x 7)
   |                ^
```

Source names are the file path, `<stdin>`, `<eval>`, or `<repl>`. `--no-color`
prints the same text without color. Evaluation runs on a dedicated thread with a
large stack so the call-depth limit reports an error instead of aborting.

## I/O

Ordinary native procedures. `display`, `write`, and `newline` return `()`.

#### `display`

Prints each argument with nothing between them. Strings are raw text; other
values use their usual printed form. Stdout is flushed.

```scheme
(display "hello " "world")  ; hello world
(display 1 2)               ; 12
```

#### `write`

Like `display`, but strings are quoted and escapes are shown.

```scheme
(write "hello\n" 1)   ; "hello\n"1
```

#### `newline`

Prints a newline. Takes no arguments.

#### `read`

Reads one line from stdin and returns it as a string, with a trailing newline
or carriage return removed. End of file returns `false`. The line is not parsed;
use `str->num` for a number. Extra arguments are ignored.

```scheme
(define n (str->num (read)))
```

#### `exit`

Flushes stdout and ends the process. No arguments means status 0; one argument
must be an integer from 0 to 255. The call does not return, so a REPL session
does not save history. Any other argument is an evaluation error and the
process keeps running.

#### `load`

Reads a string path (from the current directory) and evaluates it in the
current environment. Returns the last value, or `()` when the file is empty.
A shebang line is not stripped. When a form in the loaded file fails, the CLI
prints that file's diagnostic first (with the loaded path as the source name),
then reports a terse `load: "path" failed` error at the `(load ...)` call site.
`,load` prints the value and uses the file as source context.

```scheme
(load "examples/counter.rsc")
```

## System

| Procedure | Behavior |
| --- | --- |
| `getenv` | One string name. The value, or `false` if unset or not Unicode. |
| `clock` | No arguments. Seconds since the Unix epoch, or `0` if the clock is earlier. |
| `random` | No arguments. Uniform number in `[0, 1)`. No seed. |
| `command-line` | No arguments. The list in [Running](#running). |

## Vectors

A [`Foreign`](tutorials/foreign.md) wrapper around `RefCell<Vec<Expr>>`.
Mutable, compared by identity, printed as `<foreign: 0x…>`. `vec-push` and
`vec-set!` return `()`. An index is an integer ≥ 0; a negative index, a
non-integer, an index past the end, or `vec-pop` of an empty vector is an
error. A tracer keeps closures stored in a vector reachable. See the
[foreign object tutorial](tutorials/foreign.md).

```scheme
(define v (vec 10 20 30))
(vec-set! v 1 99)
(vec-push v 40)
(vec-get v 0)      ; 10
(vec-pop v)        ; 40
(vec-length v)     ; 3
(vec->list v)      ; (10 99 30)
(vec? v)           ; true
(list->vec '(a b))
```

| Procedure | Behavior |
| --- | --- |
| `vec?` | `true` if the argument is a vector, otherwise `false`. |
| `vec` | A new vector of the arguments, in order. `(vec)` is empty. |
| `vec-push` | Append a value. Returns `()`. |
| `vec-pop` | Remove and return the last element. |
| `vec-get` | Element at an index. |
| `vec-set!` | Replace the element at an index. Returns `()`. |
| `vec-length` | Number of elements. |
| `vec->list` | List of the elements, in order. |
| `list->vec` | New vector from a list. |

## Dicts

A [`Foreign`](tutorials/foreign.md) wrapper around `RefCell<BTreeMap<Key, Expr>>`.
Mutable, compared by identity, printed as `<foreign: 0x…>`. Keys may be
booleans, numbers, strings, or symbols; lists, procedures, and foreign values
are rejected. `-0.0` and `0.0` are the same key; `NaN` is not a valid key.
`dict-keys` and `dict->list` use a deterministic key order. `dict-set!` and
`dict-remove!` return `()`. A tracer keeps closures stored as values reachable.
See the [foreign object tutorial](tutorials/foreign.md), and
[`examples/dict.rsc`](../examples/dict.rsc) / [`examples/fibonacci.rsc`](../examples/fibonacci.rsc)
for scripts that use it.

```scheme
(define d (dict "a" 1 "b" 2))
(dict-set! d "c" 3)
(dict-get d "a")           ; 1
(dict-get d "missing" 0)   ; 0
(dict-has? d "b")          ; true
(dict-remove! d "b")
(dict-length d)            ; 2
(dict-keys d)              ; ("a" "c")
(dict->list d)             ; (("a" 1) ("c" 3))
(dict? d)                  ; true
(list->dict '((x 10) (y 20)))
```

| Procedure | Behavior |
| --- | --- |
| `dict?` | `true` if the argument is a dict, otherwise `false`. |
| `dict` | A new dict from flat key/value pairs. `(dict)` is empty. Odd argument count is an error. |
| `dict-get` | Value for a key, or an optional default (else `false`). The default is evaluated only when the key is absent. |
| `dict-set!` | Insert or replace a key. Returns `()`. |
| `dict-has?` | `true` if the key is present. |
| `dict-remove!` | Remove a key if present. Returns `()`. |
| `dict-length` | Number of entries. |
| `dict-keys` | List of keys, in map order. |
| `dict->list` | List of two-element lists `((k v) ...)`, in map order. |
| `list->dict` | New dict from a list of two-element lists. Later duplicates win. |

## Examples

| Script | What it shows |
| --- | --- |
| [`backwards.rsc`](../examples/backwards.rsc) | A macro that reverses a sequence of forms. |
| [`counter.rsc`](../examples/counter.rsc) | A closure that counts with `set!`. |
| [`dict.rsc`](../examples/dict.rsc) | Word frequencies with `dict`. Shebang. |
| [`factorial.rsc`](../examples/factorial.rsc), [`factorial-tail-recursive.rsc`](../examples/factorial-tail-recursive.rsc) | Factorial, reading a number. |
| [`fibonacci.rsc`](../examples/fibonacci.rsc), [`fibonacci-tail-recursive.rsc`](../examples/fibonacci-tail-recursive.rsc) | Fibonacci, reading a number. The non-tail version memoizes with `dict`. |
| [`fizzbuzz.rsc`](../examples/fizzbuzz.rsc) | FizzBuzz from stdin. Shebang. |
| [`mandelbrot.rsc`](../examples/mandelbrot.rsc) | ASCII Mandelbrot. Optional width and height from `(command-line)`. Shebang. |
| [`points.rsc`](../examples/points.rsc) | `defrecord`, list destructuring, named `let`, `case` / `when` / `unless`, `sqrt` / `expt`. Shebang. |
| [`strings.rsc`](../examples/strings.rsc) | `str-find`, `str-split`, `str-join`, `str-replace`, trim/case, `str-repeat`, `str->list`. Shebang. |

```bash
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
cargo run -p rusche-cli -- examples/mandelbrot.rsc 80 30
cargo run -p rusche-cli -- examples/strings.rsc
cargo run -p rusche-cli -- examples/points.rsc
```

Rust projects under `examples/` belong to the embedding tutorials.

## Related docs

- [Language reference](language-reference.md) — core special forms, built-ins, and prelude
- [Embedding](tutorials/embedding.md) — the crate without `rusche-cli`
- [Native functions](tutorials/native-functions.md) / [Foreign wrappers](tutorials/foreign.md) — how a host adds procedures and types
