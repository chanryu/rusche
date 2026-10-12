# `rusche-turtle`

[`rusche-turtle`](../crates/rusche-turtle) is an example host: a desktop turtle-graphics
window with a script editor, REPL, canvas, and output log. Scripts and REPL
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
| Script / REPL | Left-panel tabs. Script holds the source + Output; REPL is the interactive prompt. Both editors use simple syntax coloring (Rusche keywords vs turtle API names) |
| Run | Script tab. Evaluate the script on the **current** session and canvas (does not wipe state) |
| Stop | Script and REPL. Cancel at the next turtle / `write` / `random` call; later commands continue from the **visible** turtle |
| Reset | Script and REPL. Fresh session, blank canvas, clear Output and REPL transcript |
| Examples | Script tab. Load a bundled demo (embedded at compile time) |
| Help | API hints (click a form to append it to the Script) |
| Clear | Under the canvas. Blank the drawing and home the turtle/pen; **keeps** definitions |
| Speed | Under the canvas. Crawl, Walk, Trot, Run, or Dash |
| Status | On the canvas: position, heading, show/hide, color (name or `#rrggbb`), size, and pen up or down |

While a script runs, the editor highlights the form whose effect is currently being drawn
(the call to `forward`, `right`, `pencolor`, …). The highlight follows the canvas animation,
not the evaluator, so it points at what you see even though the worker thread may already
be further ahead. Forms called through runtime-built calls (e.g.
`apply`) fall back to the nearest enclosing call that appears in the source.

The Script and REPL tabs share one session: after **Run**, procedures defined in the script
are callable from the REPL. Enter evaluates a complete form; Shift+Enter inserts a newline.
Up/Down recall history on a single-line input. Turtle commands from either tab animate with
the same **Speed** setting.

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
| `(clear)` | Blank the drawing and reset turtle/pen (same idea as UI **Clear**) |
| `(hideturtle)` / `(showturtle)` | Sprite visibility |

### Queries and I/O

| Form | Result |
| --- | --- |
| `(position)` | `(x y)` |
| `(heading)` | Degrees |
| `(pen?)` | Pen down? |
| `(write …)` / `(writeln …)` | Output log (Script) or REPL transcript |
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
- **Run** continues the current session; use **Reset** for a fresh evaluator, or **Clear** / `(clear)` to wipe only the drawing.
- Example host (`publish = false`), like `rusche-cli`.
