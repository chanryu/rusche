mod io;
mod repl;
mod runner;
mod vec;

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
    println!(r"              ____                  __         ");
    println!(r"             / __ \__  ____________/ /_  ___   ");
    println!(r"            / /_/ / / / / ___/ ___/ __ \/ _ \  ");
    println!(r"Welcome to / _, _/ /_/ (__  ) /__/ / / /  __/ !");
    println!(r"          /_/ |_|\__,_/____/\___/_/ /_/\___/   ");
}
