# `rusche-turtle`

[`rusche-turtle`](../crates/rusche-turtle) is an example host: a desktop turtle-graphics
window with a script editor, interactive REPL, canvas, and output log. Scripts and REPL
expressions run on a worker-thread `Evaluator`; turtle commands stream to the UI over a
channel.

## Running

```bash
cargo run -p rusche-turtle
cargo run -p rusche-turtle -- path/to/script.rsc   # optional: load into the editor
```

Bundled demos ship inside the binary (`src/examples/*.rsc` via `include_str!`); pick them from **Examples** in the UI.

| Control | Role |
| --- | --- |
| Script / Interactive | Left-panel tabs. Script holds the source + Output; Interactive is a REPL |
| Run | Script tab. Fresh session, then evaluate the script |
| Stop | Script and Interactive. Cancel at the next turtle / `write` / `random` call |
| Examples | Script tab. Load a bundled demo (embedded at compile time) |
| Reset | Interactive tab. Fresh session and clear the REPL transcript |
| Help | API hints (click a form to append it to the Script) |
| Clear / Speed | Under the canvas. Five paces: Crawl, Walk, Trot, Run, Dash |
| Status | On the canvas: position, heading, show/hide, color, size, and pen up or down |

While a script runs, the editor highlights the form whose effect is currently being drawn
(the call to `forward`, `right`, `pencolor`, …). The highlight follows the canvas animation,
not the evaluator, so it points at what you see even though the worker thread may already
be further ahead. Forms called through runtime-built calls (e.g.
`apply`) fall back to the nearest enclosing call that appears in the source.

The Interactive tab shares the same session as the Script tab: after **Run**, procedures
defined in the script are callable from the REPL. Enter evaluates a complete form; Shift+Enter
inserts a newline. REPL turtle commands animate with the same Speed setting as Script.

## Script API

Origin at canvas center, **y up**, heading **0° = east**, positive turns
counter-clockwise. Starts at `(0, 0)` facing north (`90°`) with pen down.

Core language: [language reference](language-reference.md). Host adds:

### Motion

| Form | Effect |
| --- | --- |
| `(forward dist)` / `(fd dist)` | Move forward; draw if pen down |
| `(back dist)` / `(bk dist)` | Move backward |
| `(left deg)` / `(lt deg)` | Turn left |
| `(right deg)` / `(rt deg)` | Turn right |
| `(goto x y)` | Absolute move (draws if pen down) |
| `(setheading deg)` / `(seth deg)` | Absolute heading |
| `(home)` | Origin, face north, no draw |

### Pen and canvas

| Form | Effect |
| --- | --- |
| `(penup)` / `(pu)`, `(pendown)` / `(pd)` | Pen up / down |
| `(pencolor name)` | `black` `white` `red` `green` `blue` `yellow` `orange` `purple` `cyan` `magenta` `gray` `brown` |
| `(pencolor r g b)` | RGB `0…255` |
| `(pensize w)` | Stroke width |
| `(clear)` | Clear and reset |
| `(hideturtle)` / `(showturtle)` | Sprite visibility |

### Queries and I/O

| Form | Result |
| --- | --- |
| `(position)` | `(x y)` |
| `(heading)` | Degrees |
| `(pen?)` | Pen down? |
| `(write …)` / `(writeln …)` | Output log (Script) or REPL transcript (Interactive) |
| `(random)` | Float in `[0, 1)` |
| `(repeat n body…)` | Loop `body` `n` times |

```scheme
(pencolor "blue")
(repeat 36
  (repeat 4 (forward 100) (right 90))
  (right 10))
```

Demo sources: [`crates/rusche-turtle/src/examples/`](../crates/rusche-turtle/src/examples).

## Limitations

- **Stop** only interrupts at turtle natives, `write`/`writeln`, or `random`.
- **Run** and **Reset** start a fresh session; the REPL continues in that session afterward.
- Example host (`publish = false`), like `rusche-cli`.
