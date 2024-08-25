use rusp::{
    eval::Env,
    expr::{Expr, NIL},
};

pub fn create_test_env() -> Env {
    let env = Env::new_root_env();
    env.set("t", Expr::Sym("#t".to_string()));
    env.set("f", NIL);
    env
}
