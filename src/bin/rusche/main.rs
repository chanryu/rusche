mod prelude;
mod repl;
mod runner;
mod tokenize;

use repl::run_repl;
use runner::run_file;

fn main() {
    let mut args = std::env::args().skip(1); // skip the program name

    if let Some(path) = args.next() {
        run_file(&path);
    } else {
        print_logo();
        run_repl();
    }
}

fn print_logo() {
    println!("           ┬─┐┬ ┬┌─┐┌─┐");
    println!("Welcome to ├┬┘│ │└─┐├─┘");
    println!("           ┴└─└─┘└─┘┴  !");
}
