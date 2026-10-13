//! Turtle-graphics session, canvas animator, and desktop UI for Rusche.

pub mod app;
pub mod builtin;
pub mod canvas;
pub mod examples;
pub mod highlight;
pub mod repl;
pub mod runner;
pub mod turtle;

pub use canvas::{
    format_color, format_width, span_to_byte_range, status_text, Canvas, PenState, Pose, SPEEDS,
};
pub use repl::{is_complete, Entry, ReplState};
pub use runner::Session;
pub use turtle::{Color, Origin, Segment, TurtleEvent, TurtleSnap, DEFAULT_PEN_WIDTH};
