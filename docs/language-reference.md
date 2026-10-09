# Rusche Language Reference

> This file is the source of truth for the **core language**. Prefer updating it
> here when behaviour changes; the [GitHub wiki page](https://github.com/chanryu/rusche/wiki/Rusche-Language-Reference)
> can then be refreshed from this document.

## Scope

This reference covers only what the `rusche` crate provides:

- **Evaluator forms** — `quote`, `quasiquote`, `begin`, `if`, `eval`, and `apply`, which the evaluator recognises by name; available from every `Evaluator`, including `Evaluator::new()`
- **Built-ins** — native primitives loaded by `Evaluator::with_builtin()`
- **Prelude** — macros and helpers loaded by `Evaluator::with_prelude()` / `Default`

It does **not** document the example host [`rusche-cli`](rusche-cli.md). That
application adds I/O (`display`, `read`, `newline`), a `vec` foreign type, and
Scheme-style name aliases (`number?`, `modulo`, `string-append`, …). The
`examples/*.rsc` scripts in this repository target `rusche-cli`, not a bare
`Evaluator::default()`.

## Overview

Rusche is a minimalist, zero-dependency language and library. It is not
"batteries-included": hosts add their own I/O, collections, and domain APIs via
[native functions](tutorials/native-functions.md) and
[`Foreign`](tutorials/foreign.md) objects. The core still provides a solid
foundation:

- Garbage collection
- Tail-call optimization
- Lambdas and closures
- Lexical scoping and bindings
- Macros using special forms such as quasiquote (`` ` ``), unquote (`,`), and unquote-splicing (`,@`)

Rusche is deliberately *Scheme-like*, not Scheme: it uses Scheme's syntax but keeps the core language small. See [Differences from Scheme](#differences-from-scheme) at the end of this page.

## Data Types

Rusche supports the following data types.

- `Number`: 64-bit floating point number, e.g. `1`, `-99.12`, `.5`
- `String`: Text, e.g. `"A quick brown fox"`. Supports `\"`, `\\`, `\n`, `\r`, `\t` escapes.
- `Symbol`: Name or identifier, e.g. `car`, `num?`, `+`
- `List`: `()` or a chain of pairs ending in `()`, e.g. `(1 2 3)`. Lists are immutable.
- `Procedure`: a closure, a macro, or a native procedure.
- `Foreign`: A host-defined wrapper for external Rust objects. The core language has no built-in foreign types; see the [foreign object tutorial](tutorials/foreign.md). (`rusche-cli`'s [`vec`](rusche-cli.md#vectors) is one host example.)

## Boolean Values

Rusche doesn't have a dedicated data type for boolean values. The empty list `()` is the only false value; everything else is truthy. The prelude defines `#t` as `1` and `#f` as `()`, and predicates return `1` or `()`.

## Procedure Parameters

A parameter list is a list of symbols. The last one may be a *rest parameter*, written with a `*` prefix like Ruby's or Python's splat: `*rest` binds the symbol `rest` to a list of every remaining argument.

```scheme
(define (f a b) ...)           ; exactly two arguments
(define (f a *rest) ...)       ; one or more; `rest` is a list of the others
(lambda (*args) ...)           ; any number; `args` is a list of all of them
(defmacro (m form *forms) ...) ; same syntax for macros
```

The rest parameter must be last and there can be only one. A lone `*` is an ordinary parameter name (so `(lambda (*) ...)` shadows multiplication), and a `*` elsewhere in a name is not special. A name after the `*` may not itself start or end with `*`, so the Lisp earmuff convention `*name*` cannot be mistaken for a rest parameter.

Unlike Scheme, there is no dotted syntax -- `(f a . rest)` and `(lambda args ...)` are errors.

## Tail Calls

A call in tail position does not consume stack or call depth, so loops written as tail-recursive procedures run in constant space. The tail positions are:

- the last expression of a `lambda` body (and of `define`d procedures)
- the last argument of `begin`
- either branch of `if`
- the expression that `eval` evaluates, and the call that `apply` makes
- the expansion of a macro -- so tail position carries through prelude macros such as `cond`, `and`, `or`, `let`, `let*`, and `while`

```scheme
(define (count-down n)
  (if (= n 0) 'done (count-down (- n 1))))
(count-down 1000000)  ; done -- no stack growth
```

Non-tail procedure calls are capped by the host's `max_call_depth` (default 1000); exceeding it is an error rather than a crash. Tail calls do not count.

## Special Forms and Built-in Procedures

### Primitives

The following forms and procedures are implemented in Rust. Most are native procedures bound in the root environment and can be passed around like any other value. `quote`, `quasiquote`, `begin`, `if`, `eval`, and `apply` are instead recognised by the evaluator itself: they are not values (`(proc? if)` is an undefined-symbol error), cannot be rebound, and are available even in an `Evaluator::new()` with no built-ins loaded.

#### `atom?`
  Evaluates to true (`1`) if a given expression is an atom, i.e. anything but a non-empty list. Otherwise, false (`()`).
  ```scheme
  (atom? 123)      ; 1
  (atom? "str")    ; 1
  (atom? 'sym)     ; 1
  (atom? '())      ; 1
  (atom? '(1 2 3)) ; ()
  ```

#### `apply`
  Calls a procedure with arguments taken from a list. Closures and natives receive each value without re-evaluating it; macros receive the values as unevaluated arguments. The call is made in tail position, so a procedure that ends in `(apply f ...)` is still tail-recursive.
  ```scheme
  (apply + '(1 2 3))      ; 6
  (apply car '((1 2 3)))  ; 1
  ```

#### `begin`
  Evaluates its arguments in order and returns the value of the last one. `begin` does not create a new scope, so a `define` inside it binds in the enclosing environment. `(begin)` is `()`.
  ```scheme
  (begin (define x 1) (set! x (+ x 1)) x)  ; 2
  x                                        ; 2
  ```

#### `car`
  Returns the first element of a list.
  ```scheme
  (car '(1 2 3))   ; 1
  (car '((1 2) 3)) ; (1 2)
  (car '())        ; error
  ```

#### `cdr`
  Returns the rest of a list, excluding the first element.
  ```scheme
  (cdr '(1 2 3))   ; (2 3)
  (cdr '((1 2) 3)) ; (3)
  (cdr '(1))       ; ()
  (cdr '())        ; error
  ```

#### `cons`
  Prepends an element to a list. The second argument must be a list.
  ```scheme
  (cons 1 '(2 3))    ; (1 2 3)
  (cons '(1 2) '(3)) ; ((1 2) 3)
  (cons 1 '())       ; (1)
  (cons 1 2)         ; error -- no dotted pairs
  ```

#### `define`
  Binds a value to a symbol in the current environment.
  ```scheme
  (define x 42)    ; Binds 42 to the symbol 'x'
  ```
  You can also create a named procedure using define.
  ```scheme
  (define (add a b) (+ a b))
  (define (sum *numbers) (apply + numbers))
  (sum 1 2 3)      ; 6
  ```

#### `defmacro`
  Defines a macro. Arguments are passed unevaluated; the body produces a form that is then evaluated in the caller's environment.
  ```scheme
  (defmacro (unless condition *body)
    `(if ,condition () (begin ,@body)))

  (unless (= 1 2) 'ok)  ; ok
  ```
  Both `(defmacro (name params...) body)` and `(defmacro name (params...) body)` are accepted.

#### `eq?`, `=`
  Compares two values structurally. `=` is an alias for `eq?`.
  ```scheme
  (eq? 'a 'a)         ; 1
  (eq? 'a 'b)         ; ()
  (eq? '(1 2) '(1 2)) ; 1
  (= 1 1)             ; 1
  ```

#### `error`
  Raises an evaluation error. One or more arguments; strings contribute their raw text, other values use their printed form. Arguments are joined with a single space.
  ```scheme
  (error "bad value:" 42)  ; error: bad value: 42
  ```

#### `eval`
  Evaluates an expression in the current environment, in tail position.
  ```scheme
  (eval '(+ 1 2))  ; 3
  (define x 10)
  (eval 'x)        ; 10
  ```

#### `if`
  Conditional form that selects between two branches based on a condition. The else branch is optional; when it is omitted and the condition is false, the result is `()`.
  ```scheme
  (if (= 1 1) 'yes 'no)  ; yes
  (if (= 1 2) 'yes 'no)  ; no
  (if (= 1 2) 'yes)      ; ()
  ```

#### `lambda`
  Creates an anonymous procedure (a closure).
  ```scheme
  (define add (lambda (x y) (+ x y)))
  (add 2 3)                          ; 5
  ((lambda (a *rest) rest) 1 2 3)    ; (2 3)
  ((lambda (*args) args) 1 2 3)      ; (1 2 3)
  ```

#### `proc?`
  Evaluates to true (`1`) if the given expression is a procedure (closure, macro, or native), otherwise false (`()`).
  ```scheme
  (proc? +)              ; 1
  (proc? (lambda (x) x)) ; 1
  (proc? 1)              ; ()
  ```

#### `set!`
  Mutates the value of an already defined variable. Setting an undefined variable is an error.
  ```scheme
  (define x 5)
  (set! x 10)
  x                ; 10
  ```

#### `sym?`
  Evaluates to true (`1`) if the given expression is a symbol, otherwise false (`()`).
  ```scheme
  (sym? 'foo)  ; 1
  (sym? "foo") ; ()
  ```

#### `quote` (`'`)
  Prevents evaluation of the given expression.
  ```scheme
  (quote (1 2 3))  ; (1 2 3)
  '(1 2 3)         ; (1 2 3)
  ```

#### `quasiquote` (`` ` ``)
  Partially prevents evaluation of an expression, but allows evaluation within it via unquote. Nested quasiquotes track nesting depth so an unquote only evaluates at the matching level.
  ```scheme
  (define x 10)
  `(1 2 ,x)           ; (1 2 10)
  ``(a ,,(+ 1 2))     ; (quasiquote (a (unquote 3)))
  ```

#### `unquote` (`,`)
  Evaluates an expression within a quasiquoted expression.
  ```scheme
  (define y 20)
  `(1 ,y 3)        ; (1 20 3)
  ```

#### `unquote-splicing` (`,@`)
  Splices the result of evaluating a list within a quasiquoted list.
  ```scheme
  (define z '(4 5 6))
  `(1 2 ,@z)       ; (1 2 4 5 6)
  ```

### Prelude

The following forms and procedures are implemented in Rusche itself. Please check [prelude.rs](../src/prelude.rs) to see how they are actually implemented.

Macros: `and`, `cond` (with `else`), `defun`, `let`, `let*`, `or`, `while`

Procedures: `append`, `assoc`, `caar`, `cadr`, `cdar`, `cddr`, `filter`, `fold`, `length`, `list`, `map`, `member`, `not`, `null?`, `reverse`, `<`, `>`, `<=`, `>=`, `abs`, `min`, `max`

`and` and `or` short-circuit and return the deciding operand:
```scheme
(and 1 2 3)       ; 3
(and 1 '() 3)     ; ()
(or '() 2 3)      ; 2
(or)              ; ()
```

`list` is an ordinary procedure; `apply` is an evaluator form:
```scheme
(list 1 2 3)            ; (1 2 3)
(map list '(1 2))       ; ((1) (2))
(apply + '(1 2 3))      ; 6
(apply car '((1 2 3)))  ; 1
```

`let*` binds sequentially; each binding can use previous ones:
```scheme
(let* ((x 1) (y (+ x 2))) y)  ; 3
```

List helpers:
```scheme
(length '(a b c))                      ; 3
(filter (lambda (x) (< x 3)) '(1 2 3)) ; (1 2)
(fold + 0 '(1 2 3))                    ; 6
(member 'b '(a b c))                   ; (b c)
```

Numeric helpers and comparisons (comparisons take two or more arguments):
```scheme
(abs -3)           ; 3
(min 3 1 2)        ; 1
(max 3 1 2)        ; 3
(< 1 2 3)          ; 1
(<= 1 1 2)         ; 1
(> 3 2 1)          ; 1
```

### Number functions

#### `num?`
  Evaluates to true (`1`) if the given expression is a number, otherwise false (`()`).
  ```scheme
  (num? 123)    ; 1
  (num? "123")  ; ()
  ```

#### `num-add`, `+`
  Adds one or more numbers.
  ```scheme
  (+ 1 2 3)     ; 6
  ```

#### `num-subtract`, `-`
  Subtracts numbers in sequence.
  ```scheme
  (- 10 4 2)    ; 4
  ```

#### `num-multiply`, `*`
  Multiplies one or more numbers.
  ```scheme
  (* 2 3 4)     ; 24
  ```

#### `num-divide`, `/`
  Divides numbers in sequence.
  ```scheme
  (/ 7 2)       ; 3.5
  ```

#### `num-modulo`, `%`
  Returns the remainder of dividing two numbers. The result takes the sign of the dividend.
  ```scheme
  (% 25 7)      ; 4
  (% -7 2)      ; -1
  ```

#### `num-less`, `<`
  Compares two numbers, returns true (`1`) if the first is less than the second, otherwise false (`()`).
  ```scheme
  (< 3 5)       ; 1
  (< 10 5)      ; ()
  ```

#### `>`
  Compares numbers, returns true (`1`) if each is greater than the next, otherwise false (`()`). Implemented in the prelude via `<` on the reversed arguments.
  ```scheme
  (> 5 3)       ; 1
  (> 3 2 1)     ; 1
  (> 2 4)       ; ()
  ```

### String functions

#### `str?`
  Evaluates to true (`1`) if the given expression is a string, otherwise false (`()`).
  ```scheme
  (str? "hello")   ; 1
  (str? 123)       ; ()
  ```

#### `str-append`
  Concatenates zero or more strings.
  ```scheme
  (str-append "hello" " " "world")  ; "hello world"
  ```

#### `str-compare`
  Compares two strings lexicographically, returning `-1`, `0`, or `1`.
  ```scheme
  (str-compare "abc" "def")   ; -1
  (str-compare "xyz" "abc")   ; 1
  (str-compare "same" "same") ; 0
  ```

#### `str-length`
  Returns the number of characters in a string.
  ```scheme
  (str-length "hello")   ; 5
  (str-length "")        ; 0
  ```

#### `str-slice`
  Extracts a substring given a start index and an optional end index. Negative indices count from the end.
  ```scheme
  (str-slice "hello" 1 4)    ; "ell"
  (str-slice "world" 2)      ; "rld"
  (str-slice "example" 1 -1) ; "xampl"
  ```

### Conversion functions

#### `num->str`
  Converts a number to a string, using the same formatting as printed numbers (`1` not `1.0`).
  ```scheme
  (num->str 123)   ; "123"
  (num->str 1.5)   ; "1.5"
  ```

#### `str->num`
  Parses a string into a number if possible, otherwise returns `()`.
  ```scheme
  (str->num "123")  ; 123
  (str->num "abc")  ; ()
  ```

#### `sym->str`
  Converts a symbol to a string.
  ```scheme
  (sym->str 'foo)  ; "foo"
  ```

#### `str->sym`
  Converts a string to a symbol. Any string is accepted, including the empty string.
  ```scheme
  (str->sym "foo")  ; foo
  ```

## Differences from Scheme

- **Truthiness.** `()` is the only false value; `#t` is `1` and `#f` is `()`. There is no boolean type.
- **Equality.** `eq?` compares structurally and `=` is an alias for it.
- **Lists only.** `cons` requires a list as its second argument; there are no dotted pairs or `set-car!`/`set-cdr!`, and a lone `.` is a syntax error. Lists are immutable and shared.
- **Numbers.** All numbers are 64-bit floats.
- **Macros.** `defmacro` (unhygienic) is the macro system; there is no `syntax-rules`.
- **Names.** Type checks end in `?` (`num?`, `str?`, `sym?`, `proc?`, `atom?`). Same-type operations use a type prefix (`num-add`, `str-append`). Conversions use `type1->type2` (`num->str`, `str->num`). Scheme spellings are a [`rusche-cli`](rusche-cli.md#scheme-style-aliases) convenience, not part of the core.
- **Small surface.** `if` without an else branch and `define` return `()`; there is no named `let`, `case`, `do`, `when`, or `unless`. No characters, vectors, ports, or continuations.
- **`apply` and `eval` are syntax.** In Scheme they are procedures; in Rusche they are evaluator forms like `if`, so they cannot be passed as values or rebound. `(begin)` with no arguments is allowed and returns `()`.
- **Rest parameters** are spelled with a `*` prefix instead of Scheme's dotted syntax: `(define (f a *rest) ...)` and `(lambda (*args) ...)`. See [Procedure Parameters](#procedure-parameters).

## See also

- [`rusche-cli`](rusche-cli.md) — example host extras (I/O, `vec`, Scheme aliases)
- [Embedding tutorials](tutorials/embedding.md) — using the core crate from Rust
