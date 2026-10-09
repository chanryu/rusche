use std::cell::RefCell;

thread_local! {
    static COMMAND_LINE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub fn set_command_line(args: Vec<String>) {
    COMMAND_LINE.with(|cell| *cell.borrow_mut() = args);
}

pub fn command_line() -> Vec<String> {
    COMMAND_LINE.with(|cell| cell.borrow().clone())
}
