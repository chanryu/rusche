# `rusche-turtle`

[`rusche-turtle`](../crates/rusche-turtle) is an example host: a desktop turtle-graphics
window whose chrome is a Rusche script on [`rusche-egui`](../crates/rusche-egui). Turtle
commands still run on a worker-thread `Evaluator` shared with
[`rusche-turtle-rs`](rusche-turtle-rs.md); the canvas animator and session live in that
crate's library.

## Running

```bash
cargo run -p rusche-turtle
cargo run -p rusche-turtle -- path/to/script.rsc   # optional: load into the editor
```

The UI source is [`crates/rusche-turtle/src/ui.rsc`](../crates/rusche-turtle/src/ui.rsc).
Bundled demos come from `rusche-turtle-rs` (`Examples` in the UI).

| Control | Role |
| --- | --- |
| Script / REPL | Left-panel tabs. Script holds the source + Output; REPL is the interactive prompt |
| Run | Script tab. Evaluate the script on the **current** session and canvas |
| Stop | Cancel evaluation; later commands continue from the **visible** turtle |
| Reset | Fresh session, blank canvas, clear Output and REPL transcript |
| Examples | Load a bundled demo |
| Help | API hints (click a form to append it to the Script) |
| Clear | Under the canvas. Blank the drawing; **keeps** definitions |
| Speed | Crawl → Dash animation pace |

## Architecture

- **`ui.rsc`** rebuilds panels, editors, help, and the canvas each frame through `rusche-egui`.
- **Host natives** (`turtle-frame`, `turtle-run`, …) close over a shared [`Canvas`](../crates/rusche-turtle-rs/src/canvas.rs).
- **Worker session** evaluates turtle scripts and streams motion events; `turtle-frame` advances the animator each UI pass.

The turtle script API (`forward`, `pencolor`, `repeat`, …) matches
[`rusche-turtle-rs`](rusche-turtle-rs.md#turtle-api).

## Limitations

- Same Stop / Run / Reset semantics as `rusche-turtle-rs`.
- Example host (`publish = false`), like `rusche-cli`.
- The Rust-UI twin [`rusche-turtle-rs`](rusche-turtle-rs.md) remains available for comparison.
