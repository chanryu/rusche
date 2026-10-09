use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rusche-cli")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn script(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/scripts")
        .join(name)
}

fn example(name: &str) -> PathBuf {
    workspace_root().join("examples").join(name)
}

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NO_COLOR", "1")
        .spawn()
        .expect("spawn rusche-cli");
    if let Some(mut input) = child.stdin.take() {
        input.write_all(stdin.as_bytes()).unwrap();
    }
    child.wait_with_output().expect("wait rusche-cli")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn ok_script_exits_zero() {
    let out = run(&[script("ok.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "1\n");
    assert!(stderr(&out).is_empty());
}

#[test]
fn runtime_error_goes_to_stderr_and_exits_one() {
    let out = run(&[script("runtime_error.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    // display without newline is followed by ensure_newline before the diagnostic
    assert_eq!(stdout(&out), "hi\n");
    let err = stderr(&out);
    assert!(err.contains("error:"), "{err}");
    assert!(err.contains("car:"), "{err}");
    assert!(err.contains("^"), "{err}");
    assert!(err.contains("-->"), "{err}");
}

#[test]
fn lex_error_exits_one() {
    let out = run(&[script("lex_error.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("unterminated string literal"));
    assert!(stdout(&out).is_empty());
}

#[test]
fn incomplete_expression_exits_one() {
    let out = run(&[script("incomplete.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(
        err.contains("unclosed") || err.contains("incomplete"),
        "{err}"
    );
}

#[test]
fn shebang_is_skipped() {
    let out = run(&[script("shebang.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "shebang-ok\n");
}

#[test]
fn tab_caret_aligns_with_tab_padding() {
    let out = run(&[script("tabs.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    // The caret line should contain a literal tab before the carets (column after the tab).
    assert!(
        err.lines()
            .any(|line| line.contains('\t') && line.contains('^')),
        "expected tab-padded caret line, got:\n{err}"
    );
}

#[test]
fn missing_file_exits_two() {
    let out = run(&["/tmp/rusche-cli-definitely-missing.rsc"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("Failed to read file"));
}

#[test]
fn help_and_version() {
    let help = run(&["--help"], "");
    assert_eq!(help.status.code(), Some(0));
    assert!(stdout(&help).contains("Usage: rusche-cli"));

    let version = run(&["--version"], "");
    assert_eq!(version.status.code(), Some(0));
    assert!(stdout(&version).contains("rusche-cli"));
}

#[test]
fn eval_flag() {
    let out = run(&["-e", "(+ 1 2)"], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).trim(), "3");
}

#[test]
fn unknown_flag_exits_two() {
    let out = run(&["--not-a-real-flag"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("unknown option"));
}

#[test]
fn stdin_script() {
    let out = run(&[], "(display \"from-stdin\") (newline)\n");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "from-stdin\n");
}

#[test]
fn dash_means_stdin() {
    let out = run(&["-"], "(display 42) (newline)\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "42\n");
}

#[test]
fn no_prelude_hides_plus() {
    let out = run(&["--no-prelude", "-e", "(+ 1 2)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("undefined symbol") || stderr(&out).contains("+"));
}

#[test]
fn command_line_builtin() {
    let out = run(
        &["-e", "(begin (display (car (command-line))) (newline))"],
        "",
    );
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert!(stdout(&out).contains("rusche-cli"));
}

#[test]
fn command_line_with_script_args() {
    // Hand-rolled parser: after FILE, remaining args are script args.
    // Use -e via a tiny temp approach: run with a file that prints command-line.
    let dir = tempfile_dir();
    let path = dir.join("args.rsc");
    std::fs::write(&path, "(display (command-line)) (newline)\n").unwrap();
    let out = run(&[path.to_str().unwrap(), "alpha", "beta"], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    let printed = stdout(&out);
    assert!(printed.contains("alpha"));
    assert!(printed.contains("beta"));
}

#[test]
fn vec_and_write_builtins() {
    let out = run(
        &[
            "-e",
            r#"(begin
                (define v (vec 1 2 3))
                (display (vec-length v))
                (newline)
                (vec-set! v 1 9)
                (display (vec-get v 1))
                (newline)
                (write "hi")
                (newline)
                (display (vec? v))
                (newline))"#,
        ],
        "",
    );
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "3\n9\n\"hi\"\ntrue\n");
}

#[test]
fn dict_builtins() {
    let out = run(
        &[
            "-e",
            r#"(begin
                (define d (dict "a" 1 "b" 2))
                (display (dict-length d))
                (newline)
                (dict-set! d "c" 3)
                (display (dict-get d "a"))
                (newline)
                (display (dict-get d "missing" 0))
                (newline)
                (display (dict->list d))
                (newline)
                (display (dict? d))
                (newline))"#,
        ],
        "",
    );
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "2\n1\n0\n((\"a\" 1) (\"b\" 2) (\"c\" 3))\ntrue\n"
    );
}

#[test]
fn dict_rejects_bad_key_and_odd_args() {
    let out = run(&["-e", "(dict-get (dict) '(1))"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("dict key"), "{}", stderr(&out));

    let out = run(&["-e", "(dict 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("even number"), "{}", stderr(&out));
}

#[test]
fn scheme_boolean_aliases() {
    let out = run(&["-e", "(list #t #f (eq? #t true) (eq? #f false))"], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "(true false true true)");
}

#[test]
fn exit_builtin() {
    let out = run(&["-e", "(exit 7)"], "");
    assert_eq!(out.status.code(), Some(7));
}

#[test]
fn load_builtin() {
    let dir = tempfile_dir();
    let loaded = dir.join("loaded.rsc");
    std::fs::write(&loaded, "(define loaded-value 99)\n").unwrap();
    let expr = format!(
        "(begin (load \"{}\") (display loaded-value) (newline))",
        loaded.display()
    );
    let out = run(&["-e", &expr], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "99\n");
}

#[test]
fn newline_rejects_extra_args() {
    let out = run(&["-e", "(newline 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("expected 0 arguments"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn vec_make_rejects_extra_args() {
    let out = run(&["-e", "(vec-make 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("expected 0 arguments"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn example_counter() {
    let out = run(&[example("counter.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "1\n2\n3\n");
}

#[test]
fn example_fizzbuzz() {
    let out = run(&[example("fizzbuzz.rsc").to_str().unwrap()], "15\n");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert!(stdout(&out).contains("FizzBuzz"));
    assert!(stdout(&out).contains("Enter a number"));
}

#[test]
fn example_backwards() {
    let out = run(&[example("backwards.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "tres\ndos\nuno\n");
}

#[test]
fn example_dict() {
    let out = run(&[example("dict.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("the: 3"), "{s}");
    assert!(s.contains("fox: 2"), "{s}");
    assert!(s.contains("unique: 8"), "{s}");
    assert!(s.contains("has fox? true"), "{s}");
}

#[test]
fn example_fibonacci_memoized() {
    let out = run(&[example("fibonacci.rsc").to_str().unwrap()], "10\n");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("fib(10) => 55"), "{s}");
    assert!(s.contains("cache entries: 11"), "{s}");
}

#[test]
fn depth_limit_exits_one_not_abort() {
    let dir = tempfile_dir();
    let path = dir.join("depth.rsc");
    std::fs::write(&path, "(define (inf n) (+ 1 (inf n)))\n(inf 0)\n").unwrap();
    let out = run(&[path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1), "stderr={}", stderr(&out));
    assert!(
        stderr(&out).contains("maximum call depth"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn arity_error_snapshot() {
    let out = run(&["-e", "(car (list 1) (list 2))"], "");
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("car: expected 1 argument, got 2"), "{err}");
    assert!(err.contains("--> <eval>:"), "{err}");
}

#[test]
fn load_nested_prints_inner_source() {
    let dir = tempfile_dir();
    let inner = dir.join("inner.rsc");
    std::fs::write(&inner, "(display \"in\")\n(car 99)\n").unwrap();
    let expr = format!("(load \"{}\")", inner.display());
    let out = run(&["-e", &expr], "");
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("car:"), "{err}");
    assert!(err.contains("inner.rsc"), "{err}");
    assert!(err.contains("failed"), "{err}");
}

#[test]
fn call_trace_is_printed() {
    let dir = tempfile_dir();
    let path = dir.join("deep.rsc");
    // Non-final `begin` forms keep each closure on the stack; tail calls would replace frames.
    std::fs::write(
        &path,
        "(define (a x) (begin (b x) 0))\n(define (b x) (begin (c x) 0))\n(define (c x) (car x))\n(a 42)\n",
    )
    .unwrap();
    let out = run(&[path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("in `c`") || err.contains("in `car`"), "{err}");
    assert!(err.contains("in `b`"), "{err}");
    assert!(err.contains("in `a`"), "{err}");
}

#[test]
fn sys_builtins_and_arity() {
    let out = run(
        &[
            "-e",
            r#"(begin
                (display (num? (clock)))
                (newline)
                (display (num? (random)))
                (newline)
                (display (null? (cdr (command-line))))
                (newline)
                (display (str? (or-else (getenv "PATH") "")))
                (newline))"#,
        ],
        "",
    );
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    let printed = stdout(&out);
    assert!(printed.lines().any(|l| l == "true"), "{printed}");

    let out = run(&["-e", "(clock 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("expected 0 arguments"));

    let out = run(&["-e", "(random 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("expected 0 arguments"));

    let out = run(&["-e", "(command-line 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("expected 0 arguments"));

    let out = run(&["-e", "(getenv 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("expected a string"));

    let out = run(&["-e", "(getenv \"__RUSCHE_NO_SUCH_VAR__\")"], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "false");
}

#[test]
fn vec_error_paths() {
    let out = run(
        &[
            "-e",
            r#"(begin
                (define v (vec 1 2))
                (display (vec-get v 0))
                (newline)
                (display (vec-length v))
                (newline)
                (display (vec->list v))
                (newline)
                (display (vec? (list->vec '(a b))))
                (newline)
                (vec-push v 3)
                (display (vec-pop v))
                (newline))"#,
        ],
        "",
    );
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));

    let out = run(&["-e", "(vec-pop (vec-make))"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("vector is empty"), "{}", stderr(&out));

    let out = run(&["-e", "(vec-get (vec 1) 9)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("out-of-bounds"), "{}", stderr(&out));

    let out = run(&["-e", "(vec-set! (vec 1) -1 0)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("zero or positive"),
        "{}",
        stderr(&out)
    );

    let out = run(&["-e", "(vec-get 1 0)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("vector") || stderr(&out).contains("foreign"),
        "{}",
        stderr(&out)
    );

    let out = run(&["-e", "(list->vec 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("list"), "{}", stderr(&out));
}

#[test]
fn load_and_exit_error_paths() {
    let out = run(&["-e", "(load 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("string path"), "{}", stderr(&out));

    let out = run(
        &["-e", "(load \"/tmp/rusche-definitely-missing-xyz.rsc\")"],
        "",
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("failed to read"), "{}", stderr(&out));

    let out = run(&["-e", "(exit 999)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("0-255"), "{}", stderr(&out));
}

#[test]
fn let_shape_error_and_help_on_if() {
    let out = run(&["-e", "(let (x 1) x)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("let:"), "{}", stderr(&out));
    assert!(stderr(&out).contains("binding"), "{}", stderr(&out));

    let out = run(&["-e", "(if 1 'a 'b)"], "");
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("expected `true` or `false`"), "{err}");
    assert!(err.contains("help:"), "{err}");
}

#[test]
fn dangling_paren_and_invalid_number_messages() {
    let out = run(&["-e", ")"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("no matching"), "{}", stderr(&out));

    let out = run(&["-e", "(+ 1 23abc)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("invalid number literal"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn interleaved_display_gets_newline_before_error() {
    let out = run(&["-e", "(begin (display \"hi\") (car 1))"], "");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "hi\n");
    assert!(stderr(&out).contains("error:"));
}

fn tempfile_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rusche-cli-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
