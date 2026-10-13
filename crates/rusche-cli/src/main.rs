mod builtin;
mod cli;
mod diagnostics;
mod host;
mod repl;

use std::io::{self, IsTerminal, Read};
use std::process::ExitCode;
use std::rc::Rc;

use rusche::{tokenize_named, Evaluator, Loc, ParseError, Parser, Span};

use builtin::{load_dict_procs, load_vec_procs, IoProcs, SysProcs};
use cli::{parse_args, usage, Input, Options};
use diagnostics::{print_eval_error, print_lex_error, print_parse_error, print_pipeline_error};
use host::Host;
use repl::run_repl;

const EVAL_STACK_SIZE: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    let mut argv = std::env::args();
    let program = argv.next().unwrap_or_else(|| "rusche-cli".into());

    let opts = match parse_args(argv) {
        Ok(opts) => opts,
        Err(error) => {
            eprintln!("error: {error}");
            eprint!("{}", usage());
            return ExitCode::from(2);
        }
    };

    if opts.help {
        print!("{}", usage());
        return ExitCode::SUCCESS;
    }
    if opts.version {
        println!("rusche-cli {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if opts.no_color {
        colored::control::set_override(false);
    }

    let mut command_line = vec![program];
    match &opts.input {
        Input::File(path) => command_line.push(path.display().to_string()),
        Input::Stdin => command_line.push("-".into()),
        Input::Eval(_) | Input::Repl => {}
    }
    command_line.extend(opts.script_args.iter().cloned());

    // Evaluate on a large-stack thread so DEFAULT_MAX_CALL_DEPTH can fire as an error
    // instead of aborting with a stack overflow on debug builds.
    let opts_for_thread = opts;
    let result = std::thread::Builder::new()
        .name("rusche-eval".into())
        .stack_size(EVAL_STACK_SIZE)
        .spawn(move || run_with_options(opts_for_thread, command_line))
        .expect("failed to spawn evaluation thread")
        .join();

    match result {
        Ok(code) => code,
        Err(_) => {
            eprintln!("error: evaluation thread panicked");
            ExitCode::from(1)
        }
    }
}

fn run_with_options(opts: Options, command_line: Vec<String>) -> ExitCode {
    let host = Host::new(command_line);
    let evaluator = build_evaluator(&opts, &host);

    match opts.input {
        Input::Repl => {
            if io::stdin().is_terminal() {
                run_repl(evaluator, &host);
                ExitCode::SUCCESS
            } else {
                let mut text = String::new();
                if let Err(e) = io::stdin().read_to_string(&mut text) {
                    eprintln!("Failed to read stdin: {e}");
                    return ExitCode::from(2);
                }
                match run_source(&evaluator, &host, &text, "<stdin>") {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(()) => ExitCode::from(1),
                }
            }
        }
        Input::Stdin => {
            let mut text = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut text) {
                eprintln!("Failed to read stdin: {e}");
                return ExitCode::from(2);
            }
            match run_source(&evaluator, &host, &text, "<stdin>") {
                Ok(()) => ExitCode::SUCCESS,
                Err(()) => ExitCode::from(1),
            }
        }
        Input::File(path) => match std::fs::read_to_string(&path) {
            Ok(text) => {
                let name = path.display().to_string();
                let _source = host.enter_source_file(&path);
                match run_source(&evaluator, &host, &text, &name) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(()) => ExitCode::from(1),
                }
            }
            Err(e) => {
                eprintln!("Failed to read file at \"{}\": {e}", path.display());
                ExitCode::from(2)
            }
        },
        Input::Eval(expr) => match evaluator.eval_str_named(&expr, "<eval>") {
            Ok(result) => {
                if !result.is_nil() {
                    println!("{}", result);
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                print_pipeline_error(&host, &error, &expr, "<eval>");
                ExitCode::from(1)
            }
        },
    }
}

fn build_evaluator(opts: &Options, host: &Rc<Host>) -> Evaluator {
    let evaluator = Evaluator::with_prelude();

    if let Some(depth) = opts.max_call_depth {
        evaluator.set_max_call_depth(depth);
    }
    if let Some(threshold) = opts.gc_threshold {
        evaluator.set_gc_threshold(threshold);
    }

    evaluator
        .install(IoProcs(Rc::clone(host)))
        .install(SysProcs(Rc::clone(host)))
        .install(load_vec_procs)
        .install(load_dict_procs);
    evaluator
}

fn run_source(
    evaluator: &Evaluator,
    host: &Rc<Host>,
    text: &str,
    source_name: &str,
) -> Result<(), ()> {
    let (body, loc) = strip_shebang(text);
    let tokens = match tokenize_named(body, Some(loc), Some(source_name)) {
        Ok(tokens) => tokens,
        Err(error) => {
            print_lex_error(host, &error, text, source_name);
            return Err(());
        }
    };

    let mut parser = Parser::with_tokens(tokens);
    loop {
        match parser.parse() {
            Ok(None) => return Ok(()),
            Ok(Some(expr)) => match evaluator.eval(&expr) {
                Ok(_) => {}
                Err(e) => {
                    print_eval_error(host, &e, text, source_name);
                    return Err(());
                }
            },
            Err(error @ ParseError::IncompleteExpr(_)) => {
                let begin_loc = error.span().begin;
                let end_line = text.lines().count().saturating_sub(1);
                let end_col = text.lines().last().map(|l| l.chars().count()).unwrap_or(0);
                let span = if begin_loc.line < text.lines().count() {
                    let widened = Span::new(
                        begin_loc,
                        Loc::new(end_line, end_col.max(begin_loc.column + 1)),
                    );
                    Some(widened.with_source_from(error.span()))
                } else {
                    Some(error.span())
                };
                print_parse_error(host, &error, text, source_name, span);
                return Err(());
            }
            Err(error) => {
                print_parse_error(host, &error, text, source_name, None);
                return Err(());
            }
        }
    }
}

fn strip_shebang(text: &str) -> (&str, Loc) {
    if text.starts_with("#!") {
        if let Some(pos) = text.find('\n') {
            (&text[pos + 1..], Loc::new(1, 0))
        } else {
            ("", Loc::new(1, 0))
        }
    } else {
        (text, Loc::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli::{Input, Options};

    fn default_opts() -> Options {
        Options {
            input: Input::Repl,
            script_args: Vec::new(),
            max_call_depth: None,
            gc_threshold: None,
            no_color: true,
            help: false,
            version: false,
        }
    }

    #[test]
    fn strip_shebang_variants() {
        assert_eq!(
            strip_shebang("#!/bin/sh\n(+ 1 2)"),
            ("(+ 1 2)", Loc::new(1, 0))
        );
        assert_eq!(strip_shebang("#!only"), ("", Loc::new(1, 0)));
        assert_eq!(strip_shebang("(+ 1)"), ("(+ 1)", Loc::default()));
    }

    #[test]
    fn build_evaluator_and_run_source_paths() {
        colored::control::set_override(false);

        let mut opts = default_opts();
        opts.max_call_depth = Some(8);
        opts.gc_threshold = Some(None);
        let host = Host::new(vec!["rusche".into()]);
        let evaluator = build_evaluator(&opts, &host);
        assert_eq!(evaluator.eval_str("(+ 1 2)").unwrap().to_string(), "3");

        let mut opts = default_opts();
        opts.gc_threshold = Some(Some(50));
        let host = Host::new(Vec::new());
        let evaluator = build_evaluator(&opts, &host);
        assert!(run_source(&evaluator, &host, "#!/usr/bin/env rusche\n1\n", "<t>").is_ok());
        assert!(run_source(&evaluator, &host, "(car 1)", "<t>").is_err());
        assert!(run_source(&evaluator, &host, "\"unterminated", "<t>").is_err());
        assert!(run_source(&evaluator, &host, "(", "<t>").is_err());
        assert!(run_source(&evaluator, &host, ")", "<t>").is_err());
    }
}
