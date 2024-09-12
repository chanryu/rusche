mod repl;
mod run_file;
mod tokenize;

use repl::run_repl;
use run_file::run_file;

fn main() {
    if let Some(file_path) = std::env::args().skip(1).next() {
        run_file(&file_path);
    } else {
        // no filename is give. fallback to REPL
        run_repl();
    }
}
