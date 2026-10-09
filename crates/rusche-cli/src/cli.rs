use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Input {
    Repl,
    File(PathBuf),
    Stdin,
    Eval(String),
}

#[derive(Debug, PartialEq)]
pub struct Options {
    pub input: Input,
    pub script_args: Vec<String>,
    pub max_call_depth: Option<usize>,
    pub gc_threshold: Option<Option<usize>>,
    pub no_prelude: bool,
    pub no_color: bool,
    pub help: bool,
    pub version: bool,
}

#[derive(Debug, PartialEq)]
pub enum ParseCliError {
    MissingValue(&'static str),
    InvalidValue { flag: &'static str, value: String },
    UnknownFlag(String),
}

impl std::fmt::Display for ParseCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseCliError::MissingValue(flag) => write!(f, "missing value for {flag}"),
            ParseCliError::InvalidValue { flag, value } => {
                write!(f, "invalid value for {flag}: {value}")
            }
            ParseCliError::UnknownFlag(flag) => write!(f, "unknown option: {flag}"),
        }
    }
}

pub fn parse_args<I>(args: I) -> Result<Options, ParseCliError>
where
    I: IntoIterator<Item = String>,
{
    let mut opts = Options {
        input: Input::Repl,
        script_args: Vec::new(),
        max_call_depth: None,
        gc_threshold: None,
        no_prelude: false,
        no_color: false,
        help: false,
        version: false,
    };

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => opts.help = true,
            "-V" | "--version" => opts.version = true,
            "--no-prelude" => opts.no_prelude = true,
            "--no-color" => opts.no_color = true,
            "-e" | "--eval" => {
                let expr = iter
                    .next()
                    .ok_or(ParseCliError::MissingValue("--eval"))?;
                opts.input = Input::Eval(expr);
            }
            "--max-call-depth" => {
                let value = iter
                    .next()
                    .ok_or(ParseCliError::MissingValue("--max-call-depth"))?;
                let depth = value.parse::<usize>().map_err(|_| ParseCliError::InvalidValue {
                    flag: "--max-call-depth",
                    value: value.clone(),
                })?;
                opts.max_call_depth = Some(depth);
            }
            "--gc-threshold" => {
                let value = iter
                    .next()
                    .ok_or(ParseCliError::MissingValue("--gc-threshold"))?;
                if value == "off" {
                    opts.gc_threshold = Some(None);
                } else {
                    let threshold =
                        value
                            .parse::<usize>()
                            .map_err(|_| ParseCliError::InvalidValue {
                                flag: "--gc-threshold",
                                value: value.clone(),
                            })?;
                    opts.gc_threshold = Some(Some(threshold));
                }
            }
            "-" => {
                opts.input = Input::Stdin;
                opts.script_args.extend(iter);
                break;
            }
            flag if flag.starts_with('-') => {
                return Err(ParseCliError::UnknownFlag(flag.to_string()));
            }
            path => {
                opts.input = Input::File(PathBuf::from(path));
                opts.script_args.extend(iter);
                break;
            }
        }
    }

    Ok(opts)
}

pub fn usage() -> &'static str {
    "\
Usage: rusche-cli [OPTIONS] [FILE [ARGS...]]
       rusche-cli [OPTIONS] -e EXPR
       rusche-cli [OPTIONS] -

Options:
  -h, --help                 Show this help message
  -V, --version              Show version
  -e, --eval EXPR            Evaluate EXPR and exit
  --max-call-depth N         Set maximum call depth
  --gc-threshold N|off       Set GC threshold, or disable automatic GC
  --no-prelude               Start without the prelude
  --no-color                 Disable colored output

With no FILE and a TTY on stdin, starts a REPL.
With no FILE and a pipe on stdin, reads the script from stdin.
Use - to force reading from stdin.
"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_defaults() {
        let opts = parse_args(args(&[])).unwrap();
        assert_eq!(opts.input, Input::Repl);
        assert!(!opts.help);
    }

    #[test]
    fn test_file_and_script_args() {
        let opts = parse_args(args(&["script.rsc", "a", "b"])).unwrap();
        assert_eq!(opts.input, Input::File(PathBuf::from("script.rsc")));
        assert_eq!(opts.script_args, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_eval_and_flags() {
        let opts = parse_args(args(&[
            "--no-prelude",
            "--no-color",
            "--max-call-depth",
            "50",
            "--gc-threshold",
            "off",
            "-e",
            "(+ 1 2)",
        ]))
        .unwrap();
        assert_eq!(opts.input, Input::Eval("(+ 1 2)".into()));
        assert!(opts.no_prelude);
        assert!(opts.no_color);
        assert_eq!(opts.max_call_depth, Some(50));
        assert_eq!(opts.gc_threshold, Some(None));
    }

    #[test]
    fn test_unknown_flag() {
        assert!(matches!(
            parse_args(args(&["--bogus"])),
            Err(ParseCliError::UnknownFlag(_))
        ));
    }
}
