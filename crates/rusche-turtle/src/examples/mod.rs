//! Bundled demo scripts, compiled into the binary via `include_str!`.

pub struct Example {
    pub name: &'static str,
    pub source: &'static str,
}

pub const DEFAULT: &str = include_str!("squares.rsch");

pub const ALL: &[Example] = &[
    Example {
        name: "Nested squares",
        source: include_str!("squares.rsch"),
    },
    Example {
        name: "Spiral",
        source: include_str!("spiral.rsch"),
    },
    Example {
        name: "Star",
        source: include_str!("star.rsch"),
    },
    Example {
        name: "Tree",
        source: include_str!("tree.rsch"),
    },
    Example {
        name: "Color wheel",
        source: include_str!("colors.rsch"),
    },
    Example {
        name: "Koch snowflake",
        source: include_str!("koch.rsch"),
    },
];
