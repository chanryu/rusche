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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/scripts").join(name)
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
    assert_eq!(stdout(&out), "hi");
    let err = stderr(&out);
    assert!(err.contains("error:"));
    assert!(err.contains("car:"));
    assert!(err.contains("^"));
}

#[test]
fn lex_error_exits_one() {
    let out = run(&[script("lex_error.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("incomplete string"));
    assert!(stdout(&out).is_empty());
}

#[test]
fn incomplete_expression_exits_one() {
    let out = run(&[script("incomplete.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("incomplete expression"));
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
        err.lines().any(|line| line.contains('\t') && line.contains('^')),
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
    assert!(stderr(&out).contains("Undefined symbol") || stderr(&out).contains("+"));
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
    assert_eq!(stdout(&out), "3\n9\n\"hi\"\n1\n");
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
    assert!(stderr(&out).contains("expects no arguments"));
}

#[test]
fn vec_make_rejects_extra_args() {
    let out = run(&["-e", "(vec-make 1)"], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("expects no arguments"));
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
