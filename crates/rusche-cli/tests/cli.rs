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
fn newline_rejects_extra_args() {
    let dir = std::env::temp_dir().join(format!("rusche-cli-nl-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("nl.rsc");
    std::fs::write(&path, "(newline 1)\n").unwrap();
    let out = run(&[path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("expects no arguments"));
}

#[test]
fn vec_make_rejects_extra_args() {
    let dir = std::env::temp_dir().join(format!("rusche-cli-vm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("vm.rsc");
    std::fs::write(&path, "(vec-make 1)\n").unwrap();
    let out = run(&[path.to_str().unwrap()], "");
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
}

#[test]
fn example_backwards() {
    let out = run(&[example("backwards.rsc").to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(0), "stderr={}", stderr(&out));
    assert_eq!(stdout(&out), "tres\ndos\nuno\n");
}
