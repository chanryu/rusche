mod builtin;
mod cli;
mod diagnostics;
mod host;
mod repl;

use std::io::{self, IsTerminal, Read};
use std::process::ExitCode;

use rusche::{tokenize, Evaluator, Loc, ParseError, Parser, Span};

use builtin::{load_io_procs, load_scheme_aliases, load_sys_procs, load_vec_procs};
use cli::{parse_args, usage, Input, Options};
use diagnostics::{
    print_eval_error, print_lex_error, print_parse_error, print_pipeline_error,
};
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
    host::set_command_line(command_line);

    // Evaluate on a large-stack thread so DEFAULT_MAX_CALL_DEPTH can fire as an error
    // instead of aborting with a stack overflow on debug builds.
    let opts_for_thread = opts;
    let result = std::thread::Builder::new()
        .name("rusche-eval".into())
        .stack_size(EVAL_STACK_SIZE)
        .spawn(move || run_with_options(opts_for_thread))
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

fn run_with_options(opts: Options) -> ExitCode {
    let evaluator = build_evaluator(&opts);

    match opts.input {
        Input::Repl => {
            if io::stdin().is_terminal() {
                run_repl(evaluator);
                ExitCode::SUCCESS
            } else {
                let mut text = String::new();
                if let Err(e) = io::stdin().read_to_string(&mut text) {
                    eprintln!("Failed to read stdin: {e}");
                    return ExitCode::from(2);
                }
                match run_source(&evaluator, &text, "<stdin>") {
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
            match run_source(&evaluator, &text, "<stdin>") {
                Ok(()) => ExitCode::SUCCESS,
                Err(()) => ExitCode::from(1),
            }
        }
        Input::File(path) => match std::fs::read_to_string(&path) {
            Ok(text) => {
                let name = path.display().to_string();
                match run_source(&evaluator, &text, &name) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(()) => ExitCode::from(1),
                }
            }
            Err(e) => {
                eprintln!("Failed to read file at \"{}\": {e}", path.display());
                ExitCode::from(2)
            }
        },
        Input::Eval(expr) => match evaluator.eval_str(&expr) {
            Ok(result) => {
                if !result.is_nil() {
                    println!("{}", result);
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                print_pipeline_error(&error, &expr, "<eval>");
                ExitCode::from(1)
            }
        },
    }
}

fn build_evaluator(opts: &Options) -> Evaluator {
    let evaluator = if opts.no_prelude {
        Evaluator::with_builtin()
    } else {
        Evaluator::with_prelude()
    };

    if let Some(depth) = opts.max_call_depth {
        evaluator.set_max_call_depth(depth);
    }
    if let Some(threshold) = opts.gc_threshold {
        evaluator.set_gc_threshold(threshold);
    }

    load_io_procs(evaluator.context());
    load_sys_procs(evaluator.context());
    load_vec_procs(&evaluator);
    // Scheme aliases reference prelude helpers (`not`, `assoc`, `=`, …).
    if !opts.no_prelude {
        load_scheme_aliases(&evaluator);
    }
    evaluator
}

fn run_source(evaluator: &Evaluator, text: &str, source_name: &str) -> Result<(), ()> {
    let (body, loc) = strip_shebang(text);
    let tokens = match tokenize(body, Some(loc)) {
        Ok(tokens) => tokens,
        Err(error) => {
            print_lex_error(&error, text, source_name);
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
                    print_eval_error(&e, text, source_name);
                    return Err(());
                }
            },
            Err(error @ ParseError::IncompleteExpr(_)) => {
                let begin_loc = error.span().begin;
                let end_line = text.lines().count().saturating_sub(1);
                let end_col = text.lines().last().map(|l| l.chars().count()).unwrap_or(0);
                let span = if begin_loc.line < text.lines().count() {
                    Some(Span::new(
                        begin_loc,
                        Loc::new(end_line, end_col.max(begin_loc.column + 1)),
                    ))
                } else {
                    Some(error.span())
                };
                print_parse_error(&error, text, source_name, span);
                return Err(());
            }
            Err(error) => {
                print_parse_error(&error, text, source_name, None);
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
