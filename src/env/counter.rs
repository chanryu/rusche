#[cfg(debug_assertions)]
static mut ENV_COUNT: i32 = 0;

#[cfg(debug_assertions)]
pub fn increment_env_count(_event: &str) {
    unsafe {
        ENV_COUNT += 1;
        // println!("{_event}: {}", ENV_COUNT);
    }
}

#[cfg(debug_assertions)]
pub fn decrement_env_count() {
    unsafe {
        ENV_COUNT -= 1;
        // println!("Env dropped: {}", ENV_COUNT);
    }
}
