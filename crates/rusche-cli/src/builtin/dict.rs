use rusche::{
    cons, eval, eval_into_list, get_2_or_3_args, get_exact_1_arg, get_exact_2_args,
    get_exact_3_args, intern, ErrorKind, EvalContext, EvalError, EvalResult, Evaluator, Expr,
    ForeignObject, List, NIL,
};

use std::{cell::RefCell, cmp::Ordering, collections::BTreeMap, fmt, ops::Deref, rc::Rc};

pub fn load_dict_procs(evaluator: &Evaluator) {
    let env = evaluator.root_env();
    env.define_native_proc("dict?", is_dict);
    env.define_native_proc("dict", dict);
    env.define_native_proc("dict-get", dict_get);
    env.define_native_proc("dict-set!", dict_set);
    env.define_native_proc("dict-has?", dict_has);
    env.define_native_proc("dict-remove!", dict_remove);
    env.define_native_proc("dict-length", dict_length);
    env.define_native_proc("dict-keys", dict_keys);
    env.define_native_proc("dict->list", dict_to_list);
    env.define_native_proc("list->dict", list_to_dict);
}

#[derive(Debug)]
struct Dict(RefCell<BTreeMap<Key, Expr>>);

impl Deref for Dict {
    type Target = RefCell<BTreeMap<Key, Expr>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ForeignObject for Dict {
    fn type_name(&self) -> &'static str {
        "dict"
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<dict")?;
        for (key, value) in self.borrow().iter() {
            write!(f, " ({} {})", key.clone().into_expr(), value)?;
        }
        write!(f, ">")
    }

    fn trace(&self, trace: &mut dyn FnMut(&Expr)) {
        let values: Vec<Expr> = self.borrow().values().cloned().collect();
        for value in &values {
            trace(value);
        }
    }
}

/// Number key with total order; `-0.0` normalised to `0.0`, `NaN` rejected at construction.
#[derive(Clone, Copy, Debug)]
struct NumKey(f64);

impl PartialEq for NumKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0) == Ordering::Equal
    }
}

impl Eq for NumKey {}

impl PartialOrd for NumKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NumKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Bool(bool),
    Num(NumKey),
    Str(String),
    Sym(String),
}

impl Key {
    fn from_expr(proc_name: &str, expr: &Expr) -> Result<Self, EvalError> {
        if let Some(b) = expr.as_bool() {
            return Ok(Key::Bool(b));
        }
        if let Some(n) = expr.as_num() {
            if n.is_nan() {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: `NaN` cannot be used as a dict key"),
                )
                .with_span(expr.span()));
            }
            let n = if n == 0.0 { 0.0 } else { n };
            return Ok(Key::Num(NumKey(n)));
        }
        if let Some(s) = expr.as_str() {
            return Ok(Key::Str(s.to_string()));
        }
        if let Some(s) = expr.as_sym() {
            return Ok(Key::Sym(s.as_str().to_string()));
        }
        Err(EvalError::new(
            ErrorKind::Type,
            format!(
                "{proc_name}: `{expr}` is not a valid dict key (expected bool, number, string, or symbol)"
            ),
        )
        .with_span(expr.span()))
    }

    fn into_expr(self) -> Expr {
        match self {
            Key::Bool(b) => Expr::from(b),
            Key::Num(NumKey(n)) => Expr::from(n),
            Key::Str(s) => Expr::from(s),
            Key::Sym(s) => intern(s),
        }
    }
}

fn eval_into_dict(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<Dict>, EvalError> {
    let value = eval(expr, context)?;
    value.as_foreign::<Dict>().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: `{expr}` does not evaluate to a dict"),
        )
        .with_span(expr.span())
    })
}

fn is_dict(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    Ok(eval_into_dict(proc_name, arg, context).is_ok().into())
}

fn dict(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let count = args.len();
    if !count.is_multiple_of(2) {
        return Err(EvalError::new(
            ErrorKind::Arity,
            format!(
                "{proc_name}: expected an even number of arguments (key/value pairs), got {count}"
            ),
        ));
    }

    let mut map = BTreeMap::new();
    let mut iter = args.iter();
    while let Some(key_expr) = iter.next() {
        let value_expr = iter.next().expect("even argument count");
        let key = Key::from_expr(proc_name, &eval(key_expr, context)?)?;
        let value = eval(value_expr, context)?;
        map.insert(key, value);
    }
    Ok(Expr::foreign(Dict(RefCell::new(map))))
}

fn dict_get(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (dict_expr, key_expr, default_expr) = get_2_or_3_args(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let key = Key::from_expr(proc_name, &eval(key_expr, context)?)?;

    if let Some(value) = dict.borrow().get(&key).cloned() {
        return Ok(value);
    }

    match default_expr {
        Some(default_expr) => eval(default_expr, context),
        None => Ok(Expr::from(false)),
    }
}

fn dict_set(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (dict_expr, key_expr, value_expr) = get_exact_3_args(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let key = Key::from_expr(proc_name, &eval(key_expr, context)?)?;
    let value = eval(value_expr, context)?;
    dict.borrow_mut().insert(key, value);
    Ok(NIL)
}

fn dict_has(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (dict_expr, key_expr) = get_exact_2_args(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let key = Key::from_expr(proc_name, &eval(key_expr, context)?)?;
    let found = dict.borrow().contains_key(&key);
    Ok(found.into())
}

fn dict_remove(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (dict_expr, key_expr) = get_exact_2_args(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let key = Key::from_expr(proc_name, &eval(key_expr, context)?)?;
    dict.borrow_mut().remove(&key);
    Ok(NIL)
}

fn dict_length(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let dict_expr = get_exact_1_arg(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let len = dict.borrow().len();
    Ok(Expr::from(len))
}

fn dict_keys(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let dict_expr = get_exact_1_arg(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let keys = dict.borrow().keys().cloned().map(Key::into_expr).collect();
    Ok(keys)
}

fn dict_to_list(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let dict_expr = get_exact_1_arg(proc_name, args)?;
    let dict = eval_into_dict(proc_name, dict_expr, context)?;
    let list = dict
        .borrow()
        .iter()
        .map(|(key, value)| {
            Expr::from(cons(
                key.clone().into_expr(),
                cons(value.clone(), List::Nil),
            ))
        })
        .collect();
    Ok(list)
}

fn list_to_dict(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let list_expr = get_exact_1_arg(proc_name, args)?;
    let list = eval_into_list(proc_name, list_expr, context)?;

    let mut map = BTreeMap::new();
    for entry in list.iter() {
        let Some(pair) = entry.as_list() else {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: `{entry}` is not a two-element list"),
            )
            .with_span(entry.span()));
        };
        let mut iter = pair.iter();
        let Some(key_expr) = iter.next() else {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: `{entry}` is not a two-element list"),
            )
            .with_span(entry.span()));
        };
        let Some(value_expr) = iter.next() else {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: `{entry}` is not a two-element list"),
            )
            .with_span(entry.span()));
        };
        if iter.next().is_some() {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: `{entry}` is not a two-element list"),
            )
            .with_span(entry.span()));
        }
        let key = Key::from_expr(proc_name, key_expr)?;
        map.insert(key, value_expr.clone());
    }
    Ok(Expr::foreign(Dict(RefCell::new(map))))
}

#[cfg(test)]
mod tests {
    use super::super::vec::load_vec_procs;
    use super::*;
    use rusche::Evaluator;

    fn with_dict() -> Evaluator {
        let evaluator = Evaluator::with_builtin();
        evaluator.install(load_dict_procs);
        evaluator
    }

    fn eval_ok(evaluator: &Evaluator, src: &str) -> String {
        match evaluator.eval_str(src) {
            Ok(v) => v.to_string(),
            Err(e) => panic!("{src}: {e}"),
        }
    }

    fn eval_err(evaluator: &Evaluator, src: &str) -> String {
        match evaluator.eval_str(src) {
            Ok(v) => panic!("expected error, got {v}"),
            Err(e) => e.message(),
        }
    }

    #[test]
    fn dict_round_trip_and_predicates() {
        let e = with_dict();
        assert_eq!(eval_ok(&e, "(dict \"a\" 1)"), "<dict (\"a\" 1)>");
        assert_eq!(eval_ok(&e, "(dict-length (dict))"), "0");
        assert_eq!(eval_ok(&e, "(dict-length (dict \"a\" 1 \"b\" 2))"), "2");
        assert_eq!(eval_ok(&e, "(dict? (dict \"a\" 1))"), "true");
        assert_eq!(eval_ok(&e, "(dict? 1)"), "false");
        assert_eq!(
            eval_ok(&e, "(dict->list (dict \"a\" 1 \"b\" 2))"),
            "((\"a\" 1) (\"b\" 2))"
        );
        assert_eq!(
            eval_ok(&e, "(dict-keys (dict \"b\" 2 \"a\" 1))"),
            "(\"a\" \"b\")"
        );
        assert_eq!(
            eval_ok(&e, "(dict->list (list->dict '((a 1) (b 2))))"),
            "((a 1) (b 2))"
        );
    }

    #[test]
    fn dict_get_default_semantics() {
        let e = Evaluator::with_prelude();
        e.install(load_dict_procs);

        assert_eq!(eval_ok(&e, "(dict-get (dict) \"missing\")"), "false");
        assert_eq!(eval_ok(&e, "(dict-get (dict) \"missing\" 42)"), "42");
        assert_eq!(eval_ok(&e, "(dict-get (dict \"k\" false) \"k\")"), "false");
        // Default is only evaluated when the key is absent.
        assert_eq!(
            eval_ok(
                &e,
                "(begin
                   (define called 0)
                   (define (boom)
                     (set! called (+ called 1))
                     99)
                   (define d (dict \"k\" 1))
                   (define v (dict-get d \"k\" (boom)))
                   (cons v (cons called ())))"
            ),
            "(1 0)"
        );
        assert_eq!(
            eval_ok(
                &e,
                "(begin
                   (define called 0)
                   (define (boom)
                     (set! called (+ called 1))
                     99)
                   (define d (dict))
                   (define v (dict-get d \"k\" (boom)))
                   (cons v (cons called ())))"
            ),
            "(99 1)"
        );
    }

    #[test]
    fn dict_mutation_and_errors() {
        let e = with_dict();
        assert_eq!(
            eval_ok(
                &e,
                "(begin
                   (define d (dict \"a\" 1))
                   (dict-set! d \"b\" 2)
                   (dict-set! d \"a\" 9)
                   (dict-remove! d \"b\")
                   (cons (dict-has? d \"a\")
                         (cons (dict-has? d \"b\")
                               (cons (dict-get d \"a\")
                                     (cons (dict-length d) ())))))"
            ),
            "(true false 9 1)"
        );

        assert!(eval_err(&e, "(dict 1)").contains("even number"));
        assert!(eval_err(&e, "(dict-get (dict) '(1))").contains("dict key"));
        assert!(eval_err(&e, "(dict-set! (dict) (lambda () 1) 0)").contains("dict key"));
        assert!(eval_err(&e, "(list->dict 1)").contains("list"));
        assert!(eval_err(&e, "(list->dict '((a)))").contains("two-element"));
        assert!(eval_err(&e, "(list->dict '((a 1 2)))").contains("two-element"));
        assert!(eval_err(&e, "(dict-set!)").contains("expected 3"));
        assert!(eval_err(&e, "(dict-get (dict))").contains("expected"));
    }

    #[test]
    fn wrong_foreign_type_is_rejected() {
        let e = with_dict();
        e.install(load_vec_procs);
        let msg = eval_err(&e, "(dict-length (vec 1))");
        assert!(msg.contains("dict"), "{msg}");
    }

    #[test]
    fn dict_trace_keeps_closures_alive() {
        let e = Evaluator::with_prelude();
        e.install(load_dict_procs);
        e.set_gc_threshold(None);
        eval_ok(
            &e,
            "(define (make-counter)
               (define n 0)
               (lambda () (set! n (+ n 1)) n))
             (define d (dict \"c\" (make-counter)))",
        );
        e.collect_garbage();
        assert_eq!(eval_ok(&e, "((dict-get d \"c\"))"), "1");
    }

    #[test]
    fn num_key_normalises_negative_zero() {
        let e = with_dict();
        assert_eq!(
            eval_ok(
                &e,
                "(begin
                   (define d (dict -0.0 1))
                   (dict-get d 0.0))"
            ),
            "1"
        );
    }
}
