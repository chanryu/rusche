mod prelude;
mod repl;
mod runner;
mod tokenize;

use repl::run_repl;
use runner::run_file;

fn main() {
    if let Some(path) = std::env::args().skip(1).next() {
        run_file(&path);
    } else {
        run_repl();
    }
}
