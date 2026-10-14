//! Lightweight Rusche syntax coloring for the Script and REPL editors.
//!
//! Scans source linearly so incomplete / mid-edit text still gets colors (unlike
//! calling [`rusche::tokenize`], which fails on unterminated strings).

use std::ops::Range;

use egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Comment,
    String,
    Number,
    Bool,
    Keyword,
    Turtle,
    Punct,
    Ident,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub kind: Kind,
}

/// Special forms and prelude macros (`src/prelude/macros.rsc`).
const KEYWORDS: &[&str] = &[
    "define",
    "defmacro",
    "defun",
    "defrecord",
    "lambda",
    "if",
    "begin",
    "quote",
    "quasiquote",
    "unquote",
    "set!",
    "while",
    "let",
    "let*",
    "letrec",
    "and",
    "or",
    "or-else",
    "cond",
    "case",
    "when",
    "unless",
    "else",
];

/// Turtle host procs and the `repeat` macro from the turtle prelude.
const TURTLE: &[&str] = &[
    "forward",
    "fd",
    "back",
    "bk",
    "left",
    "lt",
    "right",
    "rt",
    "goto",
    "setheading",
    "seth",
    "home",
    "penup",
    "pu",
    "pendown",
    "pd",
    "pencolor",
    "pensize",
    "clear",
    "hideturtle",
    "showturtle",
    "position",
    "heading",
    "pen?",
    "write",
    "writeln",
    "random",
    "repeat",
];

fn is_delim(ch: char) -> bool {
    ch.is_whitespace() || "();\"'`,".contains(ch)
}

/// Classify `text` into contiguous highlight spans covering the whole string.
pub fn classify(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let start = i;
        let ch = text[i..].chars().next().unwrap();
        let ch_len = ch.len_utf8();

        if ch.is_whitespace() {
            i += ch_len;
            while i < bytes.len() {
                let c = text[i..].chars().next().unwrap();
                if !c.is_whitespace() {
                    break;
                }
                i += c.len_utf8();
            }
            spans.push(Span {
                range: start..i,
                kind: Kind::Plain,
            });
            continue;
        }

        if ch == ';' {
            i += ch_len;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(Span {
                range: start..i,
                kind: Kind::Comment,
            });
            continue;
        }

        if ch == '"' {
            i += ch_len;
            let mut escaped = false;
            while i < bytes.len() {
                let c = bytes[i];
                if escaped {
                    escaped = false;
                    i += 1;
                    continue;
                }
                if c == b'\\' {
                    escaped = true;
                    i += 1;
                    continue;
                }
                if c == b'"' {
                    i += 1;
                    break;
                }
                if c == b'\n' {
                    break;
                }
                i += 1;
            }
            spans.push(Span {
                range: start..i,
                kind: Kind::String,
            });
            continue;
        }

        if ch == ',' && bytes.get(i + 1) == Some(&b'@') {
            i += 2;
            spans.push(Span {
                range: start..i,
                kind: Kind::Punct,
            });
            continue;
        }

        if "()'`,".contains(ch) {
            i += ch_len;
            spans.push(Span {
                range: start..i,
                kind: Kind::Punct,
            });
            continue;
        }

        // Symbol / number / bool atom.
        i += ch_len;
        while i < bytes.len() {
            let c = text[i..].chars().next().unwrap();
            if is_delim(c) {
                break;
            }
            i += c.len_utf8();
        }
        let atom = &text[start..i];
        let kind = classify_atom(atom);
        spans.push(Span {
            range: start..i,
            kind,
        });
    }

    spans
}

fn classify_atom(atom: &str) -> Kind {
    if atom == "true" || atom == "false" {
        return Kind::Bool;
    }
    if KEYWORDS.contains(&atom) {
        return Kind::Keyword;
    }
    if TURTLE.contains(&atom) {
        return Kind::Turtle;
    }
    if looks_like_number(atom) {
        return Kind::Number;
    }
    Kind::Ident
}

/// Matches the lexer: a leading digit is a number token even when it will not parse
/// (`23abc`), and `.5` / `+.5` are numbers only when the whole token parses (`.5abc` stays a symbol).
fn looks_like_number(text: &str) -> bool {
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    if body.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return true;
    }
    let after_dot = body.strip_prefix('.').unwrap_or("");
    body.starts_with('.')
        && after_dot.chars().next().is_some_and(|c| c.is_ascii_digit())
        && text.parse::<f64>().is_ok()
}

/// Foreground color for a kind. `default` is the editor's normal text color (idents / plain).
///
/// Soft purple / green / peach palette (light and dark), closer to current editor themes
/// than classic blue-keyword / maroon-string schemes.
pub fn color_for(kind: Kind, dark_mode: bool, default: Color32) -> Color32 {
    match kind {
        Kind::Plain | Kind::Ident => default,
        Kind::Comment => {
            if dark_mode {
                Color32::from_rgb(108, 112, 134) // muted gray-lavender
            } else {
                Color32::from_rgb(156, 160, 176)
            }
        }
        Kind::String => {
            if dark_mode {
                Color32::from_rgb(166, 227, 161) // soft green
            } else {
                Color32::from_rgb(64, 160, 43)
            }
        }
        Kind::Number => {
            if dark_mode {
                Color32::from_rgb(250, 179, 135) // peach
            } else {
                Color32::from_rgb(254, 100, 11)
            }
        }
        Kind::Bool | Kind::Keyword => {
            if dark_mode {
                Color32::from_rgb(203, 166, 247) // mauve
            } else {
                Color32::from_rgb(136, 57, 239)
            }
        }
        Kind::Turtle => {
            if dark_mode {
                Color32::from_rgb(137, 180, 250) // sky blue
            } else {
                Color32::from_rgb(30, 102, 245)
            }
        }
        Kind::Punct => {
            if dark_mode {
                Color32::from_rgb(166, 173, 200)
            } else {
                Color32::from_rgb(76, 79, 105)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<(Kind, &str)> {
        classify(text)
            .into_iter()
            .filter(|s| s.kind != Kind::Plain)
            .map(|s| (s.kind, &text[s.range]))
            .collect()
    }

    #[test]
    fn comment() {
        assert_eq!(
            kinds("; hello\n(+ 1)"),
            vec![
                (Kind::Comment, "; hello"),
                (Kind::Punct, "("),
                (Kind::Ident, "+"),
                (Kind::Number, "1"),
                (Kind::Punct, ")"),
            ]
        );
    }

    #[test]
    fn string_with_escape() {
        assert_eq!(
            kinds(r#"(writeln "hi \"there\"")"#),
            vec![
                (Kind::Punct, "("),
                (Kind::Turtle, "writeln"),
                (Kind::String, r#""hi \"there\"""#),
                (Kind::Punct, ")"),
            ]
        );
    }

    #[test]
    fn unterminated_string_still_colored() {
        let spans = classify("(writeln \"hi");
        assert!(spans
            .iter()
            .any(|s| s.kind == Kind::String && &"(writeln \"hi"[s.range.clone()] == "\"hi"));
    }

    #[test]
    fn number_and_keyword_vs_ident() {
        assert_eq!(
            kinds("(define x 3.14)"),
            vec![
                (Kind::Punct, "("),
                (Kind::Keyword, "define"),
                (Kind::Ident, "x"),
                (Kind::Number, "3.14"),
                (Kind::Punct, ")"),
            ]
        );
    }

    #[test]
    fn prelude_macros_and_lexer_numbers() {
        assert_eq!(
            kinds("(case x (let* y) (defun f) (or-else a b))"),
            vec![
                (Kind::Punct, "("),
                (Kind::Keyword, "case"),
                (Kind::Ident, "x"),
                (Kind::Punct, "("),
                (Kind::Keyword, "let*"),
                (Kind::Ident, "y"),
                (Kind::Punct, ")"),
                (Kind::Punct, "("),
                (Kind::Keyword, "defun"),
                (Kind::Ident, "f"),
                (Kind::Punct, ")"),
                (Kind::Punct, "("),
                (Kind::Keyword, "or-else"),
                (Kind::Ident, "a"),
                (Kind::Ident, "b"),
                (Kind::Punct, ")"),
                (Kind::Punct, ")"),
            ]
        );
        assert_eq!(
            kinds("23abc .5 +.5 .5abc -inf ..."),
            vec![
                (Kind::Number, "23abc"),
                (Kind::Number, ".5"),
                (Kind::Number, "+.5"),
                (Kind::Ident, ".5abc"),
                (Kind::Ident, "-inf"),
                (Kind::Ident, "..."),
            ]
        );
    }

    #[test]
    fn bools() {
        assert_eq!(
            kinds("(if true false)"),
            vec![
                (Kind::Punct, "("),
                (Kind::Keyword, "if"),
                (Kind::Bool, "true"),
                (Kind::Bool, "false"),
                (Kind::Punct, ")"),
            ]
        );
    }

    #[test]
    fn unquote_splicing_is_one_punct() {
        assert_eq!(
            kinds("`,@x"),
            vec![(Kind::Punct, "`"), (Kind::Punct, ",@"), (Kind::Ident, "x"),]
        );
    }

    #[test]
    fn turtle_vs_keyword() {
        assert_eq!(
            kinds("(define (f) (fd 10) (repeat 2 (right 90)))"),
            vec![
                (Kind::Punct, "("),
                (Kind::Keyword, "define"),
                (Kind::Punct, "("),
                (Kind::Ident, "f"),
                (Kind::Punct, ")"),
                (Kind::Punct, "("),
                (Kind::Turtle, "fd"),
                (Kind::Number, "10"),
                (Kind::Punct, ")"),
                (Kind::Punct, "("),
                (Kind::Turtle, "repeat"),
                (Kind::Number, "2"),
                (Kind::Punct, "("),
                (Kind::Turtle, "right"),
                (Kind::Number, "90"),
                (Kind::Punct, ")"),
                (Kind::Punct, ")"),
                (Kind::Punct, ")"),
            ]
        );
    }

    #[test]
    fn covers_entire_text() {
        let text = "(define (f x) ; note\n  (+ x 1))\n";
        let spans = classify(text);
        assert_eq!(spans.first().map(|s| s.range.start), Some(0));
        assert_eq!(spans.last().map(|s| s.range.end), Some(text.len()));
        for window in spans.windows(2) {
            assert_eq!(window[0].range.end, window[1].range.start);
        }
    }
}
