use std::borrow::Cow;
use std::rc::Rc;

use colored::Colorize;
use rusche::{tokenize, Env, Evaluator, LexError, ParseError, Parser};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Config, Context, Editor, Helper};

use crate::diagnostics::print_error;
use crate::host;

pub fn run_repl(evaluator: Evaluator) {
    print_logo();

    let config = Config::builder().auto_add_history(true).build();
    let helper = ReplHelper {
        highlighter: MatchingBracketHighlighter::new(),
        env: evaluator.root_env().clone(),
    };
    let mut rl = Editor::with_config(config).expect("Failed to initialize line reader!");
    rl.set_helper(Some(helper));

    let history_path = dirs_history_path();
    if let Some(path) = &history_path {
        let _ = rl.load_history(path);
    }

    loop {
        match rl.readline("repl❯ ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed.starts_with(',') {
                    if handle_meta(&line, &evaluator) {
                        break;
                    }
                    continue;
                }

                match tokenize(&line, None) {
                    Ok(tokens) => {
                        let mut parser = Parser::with_tokens(tokens);
                        loop {
                            match parser.parse() {
                                Ok(None) => break,
                                Ok(Some(expr)) => match evaluator.eval(&expr) {
                                    Ok(result) => {
                                        if !result.is_nil() {
                                            ensure_newline();
                                            println!("{}", result.to_string().green());
                                            host::set_at_column_zero(true);
                                        }
                                    }
                                    Err(error) => {
                                        ensure_newline();
                                        print_error(&error.message, &line, error.span);
                                        host::set_at_column_zero(true);
                                    }
                                },
                                Err(ParseError::IncompleteExpr(_)) => {
                                    // Validator should prevent this for single-line; treat as error.
                                    ensure_newline();
                                    print_error(&"incomplete expression", &line, None);
                                    host::set_at_column_zero(true);
                                    break;
                                }
                                Err(error) => {
                                    ensure_newline();
                                    print_error(&error, &line, Some(error.span()));
                                    host::set_at_column_zero(true);
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        ensure_newline();
                        print_error(&error, &line, Some(error.span()));
                        host::set_at_column_zero(true);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                ensure_newline();
                println!("{}", "^C".dimmed());
                host::set_at_column_zero(true);
            }
            Err(ReadlineError::Eof) => break,
            Err(error) => {
                eprintln!("{error}");
                break;
            }
        }
    }

    if let Some(path) = &history_path {
        let _ = rl.save_history(path);
    }
}

fn ensure_newline() {
    if !host::at_column_zero() {
        println!();
        host::set_at_column_zero(true);
    }
}

/// Returns true if the REPL should quit.
fn handle_meta(line: &str, evaluator: &Evaluator) -> bool {
    let body = line.trim().trim_start_matches(',');
    let mut parts = body.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };

    match cmd {
        "help" | "h" => {
            println!(
                "{}",
                "\
Meta-commands:
  ,help            Show this help
  ,load <file>     Evaluate a file in the current environment
  ,env             List root environment bindings
  ,gc              Run garbage collection and report unreachable envs
  ,quit            Exit the REPL
"
                .dimmed()
            );
        }
        "quit" | "q" | "exit" => return true,
        "env" => {
            let mut names = evaluator.root_env().names();
            names.sort();
            names.dedup();
            for name in names {
                println!("{name}");
            }
        }
        "gc" => {
            let before = evaluator.count_unreachable_envs();
            evaluator.collect_garbage();
            let after = evaluator.count_unreachable_envs();
            println!("collected; unreachable before={before}, after={after}");
        }
        "load" => {
            let Some(path) = parts.next() else {
                eprintln!("usage: ,load <file>");
                return false;
            };
            match std::fs::read_to_string(path) {
                Ok(text) => match evaluator.eval_str(&text) {
                    Ok(result) => {
                        if !result.is_nil() {
                            println!("{}", result.to_string().green());
                        }
                    }
                    Err(error) => {
                        print_error(&error, &text, error.span());
                    }
                },
                Err(e) => eprintln!("Failed to read \"{path}\": {e}"),
            }
        }
        other => eprintln!("unknown meta-command: ,{other} (try ,help)"),
    }
    false
}

fn dirs_history_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(std::path::PathBuf::from(home).join(".rusche_history"))
}

#[rustfmt::skip]
fn print_logo() {
    println!("          {}  ", r"    ____                  __       ".bold().cyan());
    println!("          {}  ", r"   / __ \__  ____________/ /_  ___ ".bold().cyan());
    println!("          {}  ", r"  / /_/ / / / / ___/ ___/ __ \/ _ \".bold().cyan());
    println!("Welcome to{} !", r" / _, _/ /_/ (__  ) /__/ / / /  __/".bold().cyan());
    println!("          {}  ", r"/_/ |_|\__,_/____/\___/_/ /_/\___/ ".bold().cyan());

    println!("\n{}", "To exit, press Ctrl + D. Type ,help for meta-commands.".dimmed());
}

struct ReplHelper {
    highlighter: MatchingBracketHighlighter,
    env: Rc<Env>,
}

impl Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix_start = line[..pos]
            .rfind(|c: char| c.is_whitespace() || "()'`,\"".contains(c))
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[prefix_start..pos];
        if prefix.is_empty() {
            return Ok((prefix_start, Vec::new()));
        }

        let mut names = self.env.names();
        names.sort();
        names.dedup();
        let candidates: Vec<Pair> = names
            .into_iter()
            .filter(|name| name.starts_with(prefix))
            .map(|name| Pair {
                display: name.clone(),
                replacement: name,
            })
            .collect();
        Ok((prefix_start, candidates))
    }
}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        if input.trim().starts_with(',') {
            return Ok(ValidationResult::Valid(None));
        }

        match tokenize(input, None) {
            Err(LexError::IncompleteString(_)) => Ok(ValidationResult::Incomplete),
            Err(_) => Ok(ValidationResult::Valid(None)),
            Ok(tokens) => {
                let mut parser = Parser::with_tokens(tokens);
                loop {
                    match parser.parse() {
                        Ok(None) => return Ok(ValidationResult::Valid(None)),
                        Ok(Some(_)) => continue,
                        Err(ParseError::IncompleteExpr(_)) => {
                            return Ok(ValidationResult::Incomplete);
                        }
                        Err(_) => return Ok(ValidationResult::Valid(None)),
                    }
                }
            }
        }
    }
}
