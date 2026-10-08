# Rusche Language Reference

> This file is the source of truth for the language. Prefer updating it here when
> behaviour changes; the [GitHub wiki page](https://github.com/chanryu/rusche/wiki/Rusche-Language-Reference)
> can then be refreshed from this document.

## Overview

Rusche is a minimalist, zero-dependency language and library. This means that Rusche is not a "batteries-included" language; to make it truly useful as an embedded scripting language, you'll need to add your own "batteries" (such as native functions and custom data types like vectors or hashmaps). However, Rusche is far from a toy language. It provides a robust foundation suitable for commercial-grade production environments. Rusche supports the following features and constructs:

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
- `Foreign`: A wrapper data type to support external objects. See [this example](../examples/rusche-cli/builtin/vec.rs) to learn more.

## Boolean Values

Rusche doesn't have a dedicated data type for boolean values. The empty list `()` is the only false value; everything else is truthy. The prelude defines `#t` as `1` and `#f` as `()`, and predicates return `1` or `()`.

## Procedure Parameters

Parameter lists use Scheme syntax. A rest parameter, written after a dot, receives every remaining argument as a list; a bare symbol in place of the list receives all arguments.

```scheme
(define (f a b) ...)            ; exactly two arguments
(define (f a . rest) ...)       ; one or more; `rest` is a list of the others
(lambda args ...)               ; any number; `args` is a list of all of them
(defmacro (m form . forms) ...) ; same syntax for macros
```

Non-tail procedure calls are capped by the host's `max_call_depth` (default 1000); exceeding it is an error rather than a crash. Tail calls do not count.

## Special Forms and Built-in Procedures

### Primitives

The following forms and procedures are implemented via native functions.

#### `atom?`
  Evaluates to true (`1`) if a given expression is an atom, i.e. anything but a non-empty list. Otherwise, false (`()`).
  ```scheme
  (atom? 123)      ; 1
  (atom? "str")    ; 1
  (atom? 'sym)     ; 1
  (atom? '())      ; 1
  (atom? '(1 2 3)) ; ()
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
  (define (sum . numbers) (apply + numbers))
  (sum 1 2 3)      ; 6
  ```

#### `defmacro`
  Defines a macro. Arguments are passed unevaluated; the body produces a form that is then evaluated in the caller's environment.
  ```scheme
  (defmacro (unless condition . body)
    `(if ,condition () (begin ,@body)))

  (unless (= 1 2) (display "Not equal"))  ; Prints "Not equal"
  ```
  Both `(defmacro (name . params) body)` and `(defmacro name params body)` are accepted.

#### `eq?`, `=`
  Compares two values structurally. `=` is an alias for `eq?`.
  ```scheme
  (eq? 'a 'a)         ; 1
  (eq? 'a 'b)         ; ()
  (eq? '(1 2) '(1 2)) ; 1
  (= 1 1)             ; 1
  ```

#### `eval`
  Evaluates an expression in the current environment.
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
  ((lambda (a . rest) rest) 1 2 3)   ; (2 3)
  ((lambda args args) 1 2 3)         ; (1 2 3)
  ```

#### `set!`
  Mutates the value of an already defined variable. Setting an undefined variable is an error.
  ```scheme
  (define x 5)
  (set! x 10)
  x                ; 10
  ```

#### `quote` (`'`)
  Prevents evaluation of the given expression.
  ```scheme
  (quote (1 2 3))  ; (1 2 3)
  '(1 2 3)         ; (1 2 3)
  ```

#### `quasiquote` (`` ` ``)
  Partially prevents evaluation of an expression, but allows evaluation within it via unquote.
  ```scheme
  (define x 10)
  `(1 2 ,x)        ; (1 2 10)
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

Macros: `and`, `begin`, `cond` (with `else`), `defun`, `let`, `or`, `while`

Procedures: `append`, `apply`, `assoc`, `caar`, `cadr`, `cdar`, `cddr`, `list`, `map`, `not`, `null?`, `pair`, `reverse`, `subst`, `>`, `<=`, `>=`

`and` and `or` short-circuit and return the deciding operand:
```scheme
(and 1 2 3)       ; 3
(and 1 '() 3)     ; ()
(or '() 2 3)      ; 2
(or)              ; ()
```

`list` and `apply` are ordinary procedures:
```scheme
(list 1 2 3)            ; (1 2 3)
(map list '(1 2))       ; ((1) (2))
(apply + '(1 2 3))      ; 6
(apply car '((1 2 3)))  ; 1
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
  Compares two numbers, returns true (`1`) if the first is greater than the second, otherwise false (`()`). Implemented in the prelude as `(define (> a b) (< b a))`.
  ```scheme
  (> 5 3)       ; 1
  (> 2 4)       ; ()
  ```

#### `num-parse`
  Parses a string into a number if possible, otherwise returns `()`.
  ```scheme
  (num-parse "123")  ; 123
  (num-parse "abc")  ; ()
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

## `rusche-cli`

The example interpreter [`rusche-cli`](../examples/rusche-cli) adds, on top of the core language:

- I/O: `display` (strings are printed without quotes), `newline`, `read`
- A `vec` foreign type: `vec?`, `vec-make`, `vec-push`, `vec-pop`, `vec-get`
- Scheme-style aliases for the core procedures: `number?`, `modulo`, `string->number`, `string?`, `string-append`, `string-compare`, `string-length`, `substring`, `pair?` (see [scheme.rs](../examples/rusche-cli/builtin/scheme.rs))

## Differences from Scheme

- **Truthiness.** `()` is the only false value; `#t` is `1` and `#f` is `()`. There is no boolean type.
- **Equality.** `eq?` compares structurally and `=` is an alias for it.
- **Lists only.** `cons` requires a list as its second argument; there are no dotted pairs or `set-car!`/`set-cdr!`. Lists are immutable and shared.
- **Numbers.** All numbers are 64-bit floats.
- **Macros.** `defmacro` (unhygienic) is the macro system; there is no `syntax-rules`.
- **Names.** Core procedures use short names (`num?`, `str-append`, `atom?`, `%`, ...) rather than the Scheme ones; `rusche-cli` provides the Scheme spellings as aliases.
- **Small surface.** `<`, `>`, `<=`, `>=` are binary; `if` without an else branch and `define` return `()`; there is no `let*`, named `let`, `case`, `do`, `when`, or `unless`. No characters, vectors, ports, or continuations.
- **Rest parameters** do use Scheme syntax: `(define (f a . rest) ...)` and `(lambda args ...)`.
