# `rusche-cli`

[`rusche-cli`](../crates/rusche-cli) is an **example host application**, not part of
the core `rusche` crate. It is a standalone interpreter: a REPL, a file runner,
and a few batteries the library leaves to the host.

The [language reference](language-reference.md) documents only the core language
(built-ins and prelude). Everything in the procedure sections below is added by
`rusche-cli` when it starts.

## Installation

```bash
cargo install --path crates/rusche-cli
# or run from the workspace:
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
```

`cargo install` puts the `rusche-cli` binary on your `PATH`, which is what a
shebang of `#!/usr/bin/env rusche-cli` needs.

## What it loads

On startup the program builds an evaluator and then registers host procedures.

| Layer | Source | Contents |
| --- | --- | --- |
| Core built-ins + prelude | `rusche` crate | See [language reference](language-reference.md). Built-ins are always loaded. The prelude is omitted with `--no-prelude`. |
| I/O procedures | [`builtin/io.rs`](../crates/rusche-cli/src/builtin/io.rs) | `display`, `write`, `newline`, `read`, `exit`, `load` |
| System procedures | [`builtin/sys.rs`](../crates/rusche-cli/src/builtin/sys.rs) | `getenv`, `clock`, `random`, `command-line` |
| Vector foreign type | [`builtin/vec.rs`](../crates/rusche-cli/src/builtin/vec.rs) | `vec?`, `vec-make`, `vec`, `vec-push`, `vec-pop`, `vec-get`, `vec-set!`, `vec-length`, `vec->list`, `list->vec` |
| Scheme-style aliases | [`builtin/scheme.rs`](../crates/rusche-cli/src/builtin/scheme.rs) | `number?`, `modulo`, `string-append`, `vector?`, … Omitted with `--no-prelude`. |

I/O, system, and vector procedures are always registered. Scheme aliases are
ordinary bindings evaluated in the root environment, so they need the prelude
(`not`, `assoc`, `=`, and the rest).

Scripts under [`examples/*.rsc`](../examples) are written for this host: they
use `display`, `read`, `newline`, and the Scheme aliases (`modulo`,
`string->number`, …).

## Running

```text
Usage: rusche-cli [OPTIONS] [FILE [ARGS...]]
       rusche-cli [OPTIONS] -e EXPR
       rusche-cli [OPTIONS] -
```

Options may appear in any order, and `-e` may sit among them. A second `-e`
replaces the first. The first file path or `-` ends option parsing: that becomes
the script, every argument after it is a script argument, and an earlier `-e`
is discarded. A `--` argument is an unknown option. `--flag=value` is also
unknown: the value is a separate argument.

| Command | What runs |
| --- | --- |
| `rusche-cli` | REPL when stdin is a terminal. When stdin is a pipe, the pipe is the script. |
| `rusche-cli FILE ARGS...` | The file. Every argument after `FILE` is a script argument, including ones that look like options. |
| `rusche-cli - ARGS...` | The script is read from stdin. Arguments after `-` are script arguments. |
| `rusche-cli -e EXPR` | `EXPR` is evaluated. A file or `-` later on the command line replaces this and becomes the script. |

```bash
rusche-cli --no-color examples/counter.rsc
rusche-cli examples/mandelbrot.rsc 80 30          # 80 and 30 are script args
rusche-cli examples/counter.rsc --no-color        # --no-color is a script arg
rusche-cli -e '(+ 1 2)'
echo '(+ 1 2)' | rusche-cli
echo '(command-line)' | rusche-cli - a b
```

The arguments are parsed as a whole before anything runs. An unknown option or
a missing value exits 2 even when `--help` is also present. After a successful
parse, `--help` is handled first, then `--version`. Either one prints and exits
before a script or `-e` runs. When both are present, help wins.

A leading `#!` line is skipped only for the script passed on the command line
and for a script read from stdin. `(load)` and `,load` evaluate the file as
written, shebang included. A file that is only a shebang line is an empty
program.

### What is printed

| Mode | Expression values |
| --- | --- |
| REPL | Each value other than `()` is printed on its own line. |
| File, pipe, or `-` | Values are discarded. Output comes from `display`, `write`, and `newline`. |
| `-e` | The last value is printed, unless it is `()`. |

Every top-level form runs, in order. `-e` can contain more than one form; only
the last value is printed. An empty script succeeds and prints nothing.

### The `command-line` list

`(command-line)` returns a list of strings. The first element is the program
path (`argv[0]`). What follows depends on how the process was started:

| How it was started | Result |
| --- | --- |
| REPL | `(program)` |
| `rusche-cli FILE ARGS...` | `(program FILE ARGS...)` |
| `rusche-cli - ARGS...` | `(program "-" ARGS...)` |
| A pipe with no file and no `-` | `(program)` |
| `-e` | `(program)` |

`-e` does not take script arguments: a later non-option is treated as a file
and replaces the expression.

### Options

| Option | Meaning |
| --- | --- |
| `-h`, `--help` | Print usage to stdout and exit 0. |
| `-V`, `--version` | Print `rusche-cli` and the package version to stdout and exit 0. |
| `-e`, `--eval EXPR` | Evaluate `EXPR` and exit. |
| `--max-call-depth N` | Set the maximum call depth to the non-negative integer `N`. The default is 1000. Tail calls do not count. See [Tail calls](language-reference.md#tail-calls). |
| `--gc-threshold N` | Run automatic garbage collection once the number of live environments reaches `N` (default 10000). |
| `--gc-threshold off` | Leave automatic collection off. `,gc` in the REPL still collects. |
| `--no-prelude` | Start from `Evaluator::with_builtin()`: core procedures, plus I/O, system, and `vec`. The prelude (`cond`, `let`, `+`, …) and the Scheme aliases are absent. |
| `--no-color` | Disable colored output: the banner, REPL values, meta-command help, and diagnostics. |

A missing value, a non-integer `N`, or an unknown option prints `error:` and
the usage text to stderr and exits 2.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. The REPL also exits 0 after Ctrl+D or `,quit`, including when individual entries failed. |
| `1` | Lex, parse, or evaluation error in a script, a stdin script, or `-e`. |
| `2` | Usage error, or failure to read the script file or stdin. |
| `(exit n)` | The process ends immediately with code `n`. |

`(exit)` uses `0` when called with no arguments. The code must be an integer
from 0 to 255; anything else is an evaluation error. Stdout is flushed first.
Because this ends the process, a call from the REPL does not write the history
file. See [`exit`](#exit).

Diagnostics go to stderr. Script output goes to stdout.

## REPL

```bash
cargo run -p rusche-cli
```

A terminal, with no file, no `-`, and no `-e`, starts the REPL. It prints a short banner,
then the hint `To exit, press Ctrl + D. Type ,help for meta-commands.` The
prompt is `repl❯ `.

### Editing

- An unfinished form or an unfinished string stays open for another line. A finished line can hold several forms; each one is evaluated.
- A blank line is ignored.
- Tab completes the identifier at the cursor against names bound in the root environment. The prefix runs back to whitespace or to `(`, `)`, `'`, a backtick, `,`, or `"`.
- Matching brackets are highlighted.
- History is stored in `$HOME/.rusche_history` when `HOME` is set. It is loaded on startup and written on the way out (Ctrl+D or `,quit`).
- Ctrl+C drops the current input and prints `^C`. Ctrl+D exits.

### Results

A result of `()` is not printed. Any other result is printed in green.

`display` and `write` share the terminal with the prompt. If that output did
not end in a newline, the next prompt continues on the same line, and a newline
is inserted before the next printed result.

`(read)` at the REPL consumes the next line from the terminal as data. That
line is a string result, not the next expression.

### Meta-commands

A line whose first non-whitespace character is `,` is a meta-command. It is not
evaluated as Rusche.

| Command | Action |
| --- | --- |
| `,help`, `,h` | Show meta-command help. |
| `,load FILE` | Read `FILE` and evaluate it in the current environment. The last value is printed when it is not `()`. Parse and evaluation errors use the file as source context. |
| `,env` | Print every name bound in the root environment, in sorted order. |
| `,gc` | Collect garbage and print `collected; unreachable before=B, after=A`. |
| `,quit`, `,q`, `,exit` | Leave the REPL. |

An unknown command prints `unknown meta-command: ,name (try ,help)`.
`,load` with no path prints `usage: ,load <file>`.

`,load` and [`load`](#load) both evaluate a file in the current environment.
`,load` prints the last value and reports errors against the file text. `load`
returns the last value to the caller, and its errors carry no source span.

## Diagnostics

Errors are printed to stderr:

```text
error: 7 is not a symbol.
  1| (define plus
  2|     (lambda (x 7)
   |                ^
```

The message is prefixed with `error:`. When the error has a source span, a few
surrounding lines are shown with a caret under the span. A tab in the source
line is kept in the caret's padding, so the caret stays under the right column.
A span taller than four lines keeps the first three lines and the last line,
with `...` in between. An error with no span is the message alone.

`--no-color` prints the same text without color. In the REPL the offending
input line is the source. For a script, the source is the file (the shebang
line is still in that text, and later lines keep their real line numbers).

## I/O

These are ordinary native procedures. Another host can omit them or replace
them. `display`, `write`, and `newline` return `()`.

#### `display`

Prints each argument with nothing between them. Strings are written as raw
text. Every other value uses its usual printed form (`1`, `true`, `(1 2)`,
`<foreign: 0x…>`). Stdout is flushed.

```scheme
(display "hello " "world")  ; hello world
(display 1 2)               ; 12
(display '(a b))            ; (a b)
```

#### `write`

Prints each argument with nothing between them, using the usual printed form
for every value, strings included. Strings are quoted and escapes are shown.
Stdout is flushed.

```scheme
(write "hello\n" 1)   ; "hello\n"1
(display "hello\n")   ; hello
```

#### `newline`

Prints a newline. It takes no arguments.

```scheme
(display "hi") (newline)
```

#### `read`

Reads one line from stdin and returns it as a string. A trailing newline or
carriage return is removed. At end of file the result is `false`. The line is
not parsed as Rusche; turn a numeric line into a number with `string->number`
(or `str->num`). Extra arguments are ignored. A read error is an evaluation
error.

```scheme
(display "n: ")
(define n (string->number (read)))
```

In the REPL this takes the next terminal line. See [Results](#results).

#### `exit`

Flushes stdout and ends the process. With no arguments the status is 0. With
one argument it must be an integer from 0 to 255.

```scheme
(exit)      ; status 0
(exit 2)    ; status 2
```

The call does not return. From the REPL it skips saving history. A non-integer
or a number outside 0–255 is an evaluation error and the process keeps running.

#### `load`

Reads the file at a string path and evaluates every top-level form in the
current environment. The result is the last value, or `()` if the file is
empty. The path is resolved from the process's current directory. A shebang
line is left in the source and tokenized with the rest of the file.

```scheme
(load "examples/counter.rsc")
```

If the path is not a string, the file cannot be read, or a form fails to parse
or evaluate, `load` raises an error. The message includes the path. The error
has no source span, so the diagnostic is the message alone. Definitions in the
file stay in the environment that called `load`.

## System

#### `getenv`

Returns the value of an environment variable as a string. The argument is a
string name. The result is `false` when the variable is unset or its value is
not Unicode.

```scheme
(getenv "HOME")
(getenv "NO_SUCH_VARIABLE")  ; false
```

#### `clock`

Returns the number of seconds since the Unix epoch, as a number. It takes no
arguments. If the system clock is before the epoch, the result is `0`.

```scheme
(clock)
```

#### `random`

Returns a uniform random number in the half-open range `[0, 1)`. It takes no
arguments. There is no way to set the seed.

```scheme
(random)
```

#### `command-line`

Returns the process command line as a list of strings. It takes no arguments.
The shape of the list depends on how the program was started; see
[The `command-line` list](#the-command-line-list).

```scheme
;; rusche-cli examples/mandelbrot.rsc 80 30
(define args (cddr (command-line)))  ; ("80" "30")
```

## Vectors

`rusche-cli` wraps `RefCell<Vec<Expr>>` as a [`Foreign`](tutorials/foreign.md)
object. Vectors are mutable and compared by identity: two vectors are `eq?`
when they are the same object, so a vector stored in two places shares its
updates.

A vector prints as `<foreign: 0x…>`. Use `vec->list`, or `display` the
elements, to see the contents.

```scheme
(define v (vec-make))
(vec-push v 1)
(vec-push v 2)
(vec-get v 0)    ; 1
(vec-pop v)      ; 2
(vec? v)         ; true

(define w (vec 10 20 30))
(vec-length w)   ; 3
(vec-set! w 1 99)
(vec->list w)    ; (10 99 30)
(list->vec '(a b c))
```

`vec-push` and `vec-set!` return `()`. Indexes are zero-based. An index must be
an integer greater than or equal to zero; a negative index, a non-integer, or
an index past the end is an error. `vec-pop` on an empty vector is an error.

A garbage-collection tracer is registered for this type, so a closure stored
only inside a vector stays reachable. See the
[foreign object tutorial](tutorials/foreign.md) for the general pattern.

#### `vec?`

Evaluates to `true` if the argument is a vector, otherwise `false`.

```scheme
(vec? (vec 1))  ; true
(vec? '(1))     ; false
```

#### `vec-make`

Returns a new empty vector. It takes no arguments. The Scheme alias
`make-vector` is this procedure, called as `(make-vector)`.

```scheme
(vec-make)
(vec-length (vec-make))  ; 0
```

#### `vec`

Returns a new vector holding the arguments, in order. Zero arguments produce
an empty vector.

```scheme
(vec->list (vec 'a 'b 'c))  ; (a b c)
```

#### `vec-push`

Appends a value to a vector and returns `()`.

```scheme
(define v (vec 1))
(vec-push v 2)
(vec->list v)  ; (1 2)
```

#### `vec-pop`

Removes and returns the last element. An empty vector is an error.

```scheme
(vec-pop (vec 1 2))  ; 2
```

#### `vec-get`

Returns the element at an index.

```scheme
(vec-get (vec 'a 'b 'c) 1)  ; b
```

#### `vec-set!`

Replaces the element at an index and returns `()`.

```scheme
(define v (vec 'a 'b))
(vec-set! v 0 'z)
(vec->list v)  ; (z b)
```

#### `vec-length`

Returns the number of elements.

```scheme
(vec-length (vec 1 2 3))  ; 3
```

#### `vec->list`

Returns a list of the vector's elements, in order.

```scheme
(vec->list (vec 1 2))  ; (1 2)
```

#### `list->vec`

Returns a new vector holding the elements of a list. The argument must be a
list.

```scheme
(vec->list (list->vec '(a b)))  ; (a b)
```

## Scheme-style aliases

Core procedures keep short names (`num?`, `%`, `str-append`, `str->num`, …).
`rusche-cli` binds familiar Scheme spellings as aliases in the root environment.
They are skipped with `--no-prelude`.

| Alias | Core / host name |
| --- | --- |
| `#t` | `true` |
| `#f` | `false` |
| `number?` | `num?` |
| `modulo` | `%` |
| `string->number` | `str->num` |
| `number->string` | `num->str` |
| `string?` | `str?` |
| `string-append` | `str-append` |
| `string-compare` | `str-compare` |
| `string-length` | `str-length` |
| `substring` | `str-slice` |
| `string=?` | `(lambda (a b) (= (string-compare a b) 0))` |
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
| `list-ref` | zero-based `car` / `cdr` walk |
| `assq` | `assoc` |

`#t` and `#f` are ordinary bindings of those symbols. `#t` evaluates to `true`.
`'#t` is the symbol `#t`. Either name can be rebound.

`substring` is [`str-slice`](language-reference.md#str-slice): a start index and
an optional end index, with negative indexes counted from the end.
`string=?` is the only string comparison alias. `pair?` is true for a non-empty
list; `()` is an atom, so `(pair? '())` is `false`. `list-ref` walks the list
with `car` and `cdr`, so an index past the end fails as `car` of `()`.
`make-vector` takes no arguments.

`vec`, `vec->list`, and `list->vec` have no `vector` / `vector->list` /
`list->vector` aliases.

An embedding that wants these names can copy
[`scheme.rs`](../crates/rusche-cli/src/builtin/scheme.rs). The core library does
not define them.

## Examples

The `*.rsc` scripts under [`examples/`](../examples) target this host:

| Script | What it shows |
| --- | --- |
| [`backwards.rsc`](../examples/backwards.rsc) | A `defmacro` that reverses a sequence of forms. |
| [`counter.rsc`](../examples/counter.rsc) | A closure that keeps a count with `set!`. |
| [`factorial.rsc`](../examples/factorial.rsc) | Recursive factorial, reading a number with `read`. |
| [`factorial-tail-recursive.rsc`](../examples/factorial-tail-recursive.rsc) | The same function with an accumulator. |
| [`fibonacci.rsc`](../examples/fibonacci.rsc) | Recursive Fibonacci, reading a number with `read`. |
| [`fibonacci-tail-recursive.rsc`](../examples/fibonacci-tail-recursive.rsc) | Tail-recursive Fibonacci. |
| [`fizzbuzz.rsc`](../examples/fizzbuzz.rsc) | FizzBuzz up to a number read from stdin. Starts with a shebang. |
| [`mandelbrot.rsc`](../examples/mandelbrot.rsc) | An ASCII Mandelbrot set. Optional width and height come from `(command-line)`. Starts with a shebang. |

```bash
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
cargo run -p rusche-cli -- examples/mandelbrot.rsc 80 30
cargo run -p rusche-cli -- -e '(+ 1 2)'

# After cargo install --path crates/rusche-cli:
chmod +x examples/fizzbuzz.rsc
./examples/fizzbuzz.rsc
```

The Rust projects under `examples/` (`tutorial-embed`, `tutorial-native`,
`tutorial-foreign`, `readme-demo`) are the embedding tutorials, not scripts for
this binary.

## Related docs

- [Language reference](language-reference.md) — core special forms, built-ins, and prelude
- [Embedding tutorial](tutorials/embedding.md) — using the crate without `rusche-cli`
- [Native functions](tutorials/native-functions.md) / [Foreign wrappers](tutorials/foreign.md) — how a host adds procedures and types like I/O and `vec`
