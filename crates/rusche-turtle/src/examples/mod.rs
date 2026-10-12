//! Bundled demo scripts, compiled into the binary via `include_str!`.

pub struct Example {
    pub name: &'static str,
    pub source: &'static str,
}

pub const DEFAULT: &str = include_str!("squares.rsc");

pub const ALL: &[Example] = &[
    Example {
        name: "Nested squares",
        source: include_str!("squares.rsc"),
    },
    Example {
        name: "Spiral",
        source: include_str!("spiral.rsc"),
    },
    Example {
        name: "Star",
        source: include_str!("star.rsc"),
    },
    Example {
        name: "Tree",
        source: include_str!("tree.rsc"),
    },
    Example {
        name: "Color wheel",
        source: include_str!("colors.rsc"),
    },
    Example {
        name: "Koch snowflake",
        source: include_str!("koch.rsc"),
    },
];
