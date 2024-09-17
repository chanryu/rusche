mod prelude;
mod repl;
mod runner;
mod tokenize;

use prelude::load_prelude;
use repl::run_repl;
use runner::run_file;

use rusp::eval::EvalContext;

fn main() {
    let context = EvalContext::new();

    load_prelude(context.root_env());

    if let Some(path) = std::env::args().skip(1).next() {
        run_file(&path, &context);
    } else {
        run_repl(&context);
    }
}
