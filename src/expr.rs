use std::{any::Any, fmt, rc::Rc};

use crate::{
    list::{List, ListIter},
    proc::Proc,
    span::Span,
    symbol::Symbol,
};

pub type Foreign = Rc<dyn ForeignObject>;

/// Downcast helpers for every [`ForeignObject`]. A host does not implement this.
#[doc(hidden)]
pub trait ForeignDowncast: Any {
    /// The concrete value as [`Any`], for type checks.
    fn as_any(&self) -> &dyn Any;

    /// The same `Rc`, so [`Expr::as_foreign`] can downcast it.
    fn into_any(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: Any> ForeignDowncast for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

/// A Rust value stored in an [`Expr::Foreign`].
///
/// Default methods cover a value that holds no [`Expr`]s: it prints as
/// `<type_name>` and the garbage collector does not look inside it. Override
/// [`trace`](ForeignObject::trace) when the value stores `Expr`s (especially
/// closures), and [`fmt`](ForeignObject::fmt) to print the contents.
///
/// Implement this for a type defined in the host crate. `RefCell<T>` cannot
/// implement it when `RefCell` comes from another crate; wrap that cell in a
/// local struct.
pub trait ForeignObject: ForeignDowncast {
    /// Short name used by the default [`fmt`](ForeignObject::fmt).
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Writes the printed form, including the surrounding `<>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.type_name())
    }

    /// Reports every [`Expr`] stored inside this object.
    fn trace(&self, _trace: &mut dyn FnMut(&Expr)) {}
}

impl fmt::Debug for dyn ForeignObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Foreign({})", self.type_name())
    }
}

/// The enum that represents all expression variants in the Rusche language.
#[derive(Clone, Debug)]
pub enum Expr {
    /// A 64-bit floating number value.
    Num(f64, Option<Span>),

    /// A boolean value (`true` or `false`).
    Bool(bool, Option<Span>),

    /// A string value.
    Str(String, Option<Span>),

    /// A symbol value (interned; see [`Symbol`]).
    Sym(Symbol, Option<Span>),

    /// A procedure value. There are 3 types of procedures in Rusche:
    /// - [`Proc::Native`]: implemented in Rust
    /// - [`Proc::Closure`]: user-defined via `lambda` form
    /// - [`Proc::Macro`]: user-defined via `defmacro` form
    ///
    /// Stored behind `Rc` so looking up or passing a procedure is a refcount bump.
    Proc(Rc<Proc>, Option<Span>),

    /// A list value. It can be either a cons cell or an empty list.
    List(List, Option<Span>),

    /// A foreign object value. This is used to store Rust objects in the interpreter.
    Foreign(Foreign),
}

pub const NIL: Expr = Expr::List(List::Nil, None);

impl Expr {
    /// Returns `true` if the expression is an atom.
    /// Every expresssion that is not a cons cell list is an atom.
    pub fn is_atom(&self) -> bool {
        !matches!(self, Expr::List(List::Cons(_), _))
    }

    /// Return `true` if the expression is an empty list.
    pub fn is_nil(&self) -> bool {
        matches!(self, Expr::List(List::Nil, _))
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Expr::Bool(_, span)
            | Expr::Num(_, span)
            | Expr::Str(_, span)
            | Expr::Sym(_, span)
            | Expr::Proc(_, span)
            | Expr::List(_, span) => *span,
            Expr::Foreign(_) => None,
        }
    }

    /// Returns the number, ignoring its span.
    pub fn as_num(&self) -> Option<f64> {
        match self {
            Expr::Num(value, _) => Some(*value),
            _ => None,
        }
    }

    /// Returns the boolean, ignoring its span.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Expr::Bool(value, _) => Some(*value),
            _ => None,
        }
    }

    /// Returns the string, ignoring its span.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Expr::Str(text, _) => Some(text),
            _ => None,
        }
    }

    /// Returns the symbol, ignoring its span.
    pub fn as_sym(&self) -> Option<&Symbol> {
        match self {
            Expr::Sym(name, _) => Some(name),
            _ => None,
        }
    }

    /// Returns the list, ignoring its span.
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Expr::List(list, _) => Some(list),
            _ => None,
        }
    }

    /// Returns the procedure, ignoring its span.
    pub fn as_proc(&self) -> Option<&Proc> {
        match self {
            Expr::Proc(proc, _) => Some(proc),
            _ => None,
        }
    }

    /// Downcasts a foreign object to `T`.
    ///
    /// `T` is the concrete type passed to [`Expr::foreign`].
    pub fn as_foreign<T: Any>(&self) -> Option<Rc<T>> {
        match self {
            Expr::Foreign(object) => Rc::clone(object).into_any().downcast().ok(),
            _ => None,
        }
    }

    /// Wraps `value` as [`Expr::Foreign`].
    ///
    /// Pass the value itself, not an `Rc`. [`Expr::as_foreign`] downcasts to that
    /// same type.
    pub fn foreign(value: impl ForeignObject) -> Self {
        Expr::Foreign(Rc::new(value))
    }

    /// Returns a copy of the expression with every span removed, recursively.
    pub(crate) fn without_spans(&self) -> Expr {
        match self {
            Expr::Bool(value, _) => Expr::Bool(*value, None),
            Expr::Num(value, _) => Expr::Num(*value, None),
            Expr::Str(text, _) => Expr::Str(text.clone(), None),
            Expr::Sym(name, _) => Expr::Sym(name.clone(), None),
            Expr::Proc(proc, _) => Expr::Proc(Rc::clone(proc), None),
            Expr::List(list, _) => Expr::List(list.without_spans(), None),
            Expr::Foreign(_) => self.clone(),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Bool(lhs, _), Expr::Bool(rhs, _)) => lhs == rhs,
            (Expr::Num(lhs, _), Expr::Num(rhs, _)) => lhs == rhs,
            (Expr::Str(lhs, _), Expr::Str(rhs, _)) => lhs == rhs,
            (Expr::Sym(lhs, _), Expr::Sym(rhs, _)) => lhs == rhs,
            (Expr::Proc(lhs, _), Expr::Proc(rhs, _)) => lhs == rhs,
            (Expr::List(lhs, _), Expr::List(rhs, _)) => lhs == rhs,
            (Expr::Foreign(lhs), Expr::Foreign(rhs)) => Rc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
}

/// Writes `text` as a double-quoted string literal that the lexer can read back.
fn write_escaped_str(f: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    write!(f, "\"")?;
    for ch in text.chars() {
        match ch {
            '"' => write!(f, "\\\"")?,
            '\\' => write!(f, "\\\\")?,
            '\n' => write!(f, "\\n")?,
            '\r' => write!(f, "\\r")?,
            '\t' => write!(f, "\\t")?,
            ch => write!(f, "{ch}")?,
        }
    }
    write!(f, "\"")
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Bool(true, _) => write!(f, "true"),
            Expr::Bool(false, _) => write!(f, "false"),
            Expr::Num(value, _) => write!(f, "{}", value),
            Expr::Str(text, _) => write_escaped_str(f, text),
            Expr::Sym(name, _) => write!(f, "{}", name),
            Expr::Proc(proc, _) => write!(f, "<{}>", proc.fingerprint()),
            Expr::List(list, _) => write!(f, "{}", list),
            Expr::Foreign(object) => ForeignObject::fmt(object.as_ref(), f),
        }
    }
}

impl From<List> for Expr {
    fn from(val: List) -> Self {
        Expr::List(val, None)
    }
}

impl From<Vec<Expr>> for Expr {
    fn from(value: Vec<Expr>) -> Self {
        List::from(value).into()
    }
}

impl<'a> From<ListIter<'a>> for Expr {
    fn from(value: ListIter) -> Self {
        value.cloned().collect::<Vec<_>>().into()
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Expr::Str(value, None)
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Expr::from(value.to_string())
    }
}

impl From<i32> for Expr {
    fn from(value: i32) -> Self {
        Expr::Num(value as f64, None)
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Expr::Num(value, None)
    }
}

/// Converts a Rust `bool` into a Rusche boolean: `true` / `false`.
impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Expr::Bool(value, None)
    }
}

/// Converts a length or index into a Rusche number.
///
/// Values above `2^53` are not all exactly representable as `f64`.
impl From<usize> for Expr {
    fn from(value: usize) -> Self {
        Expr::Num(value as f64, None)
    }
}

impl FromIterator<Expr> for Expr {
    fn from_iter<T: IntoIterator<Item = Expr>>(iter: T) -> Self {
        List::from_iter(iter).into()
    }
}

/// Interns a string into an [`Expr::Sym`].
///
/// # Examples
///
/// ```
/// use rusche::expr::{intern, Expr};
/// use rusche::symbol::Symbol;
///
/// let symbol = intern("foo");
/// assert_eq!(symbol, Expr::Sym(Symbol::intern("foo"), None));
/// ```
pub fn intern(name: impl AsRef<str>) -> Expr {
    Expr::Sym(Symbol::intern(name), None)
}

#[cfg(test)]
pub mod test_utils {
    use super::Expr;

    pub fn num<T: Into<f64>>(value: T) -> Expr {
        Expr::Num(value.into(), None)
    }
}

#[cfg(test)]
mod size_tests {
    use super::Expr;

    #[test]
    fn expr_size_is_at_most_64_bytes() {
        let size = std::mem::size_of::<Expr>();
        assert!(
            size <= 64,
            "Expr is {size} bytes; keep it small so clones stay cheap"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::num;
    use super::*;
    use crate::macros::list;

    #[derive(Debug)]
    struct Sample(i32);

    impl ForeignObject for Sample {
        fn type_name(&self) -> &'static str {
            "sample"
        }
    }

    #[test]
    fn test_display_nil() {
        assert_eq!(format!("{}", NIL), "()");
    }

    #[test]
    fn test_display_num() {
        assert_eq!(format!("{}", num(0)), "0");
        assert_eq!(format!("{}", num(1)), "1");
        assert_eq!(format!("{}", num(1.2)), "1.2");
        assert_eq!(format!("{}", num(2.0)), "2");
    }

    #[test]
    fn test_display_str() {
        assert_eq!(format!("{}", Expr::from("str")), "\"str\"");
        assert_eq!(
            format!("{}", Expr::from("a\"b\\c\nd\te")),
            r#""a\"b\\c\nd\te""#
        );
        assert_eq!(format!("{}", Expr::from("a\r\nb")), r#""a\r\nb""#);
    }

    #[test]
    fn test_without_spans() {
        use crate::span::{Loc, Span};

        let span = Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)));
        let proc = Rc::new(Proc::Native {
            name: "noop".into(),
            func: crate::proc::NativeFunc::new(|_, _, _| Ok(NIL)),
        });
        let spanned = [
            Expr::Bool(true, span),
            Expr::Num(1.0, span),
            Expr::Str("s".into(), span),
            Expr::Sym("x".into(), span),
            Expr::Proc(proc, span),
            Expr::List(list!(Expr::Num(2.0, span)), span),
        ];
        for expr in &spanned {
            assert!(expr.span().is_some());
            let stripped = expr.without_spans();
            assert!(stripped.span().is_none());
            assert_eq!(&stripped, expr); // PartialEq ignores spans
        }

        // Variants without spans are returned as-is.
        let foreign = Expr::foreign(Sample(1));
        assert!(foreign.without_spans().span().is_none());
        assert_eq!(foreign.without_spans(), foreign);
    }

    #[test]
    fn test_foreign_eq() {
        let a = Expr::foreign(Sample(1));
        let b = Expr::foreign(Sample(1));
        assert_eq!(a, a.clone());
        assert_ne!(a, b);
    }

    #[test]
    fn test_display_foreign_and_list_iter_from() {
        let foreign = Expr::foreign(Sample(1));
        assert_eq!(format!("{foreign}"), "<sample>");

        let list = list!(1, 2, 3);
        let from_iter: Expr = list.iter().into();
        assert_eq!(from_iter, list!(1, 2, 3).into());
    }

    #[test]
    fn test_display_sym() {
        assert_eq!(format!("{}", intern("sym")), "sym");
    }

    #[test]
    fn test_display_list_1() {
        let list = list!(0);
        assert_eq!(format!("{}", list), "(0)");
    }

    #[test]
    fn test_display_list_2() {
        let list = list!(0, 1, 2);
        assert_eq!(format!("{}", list), "(0 1 2)");
    }

    #[test]
    fn test_display_list_3() {
        let list = list!(0, "str", intern("sym"));
        assert_eq!(format!("{}", list), r#"(0 "str" sym)"#);
    }

    #[test]
    fn test_expr_from_list() {
        assert_eq!(
            format!("{}", Expr::from(list!(list!(1, 2), 3, 4, intern("abc")))),
            "((1 2) 3 4 abc)"
        );
    }

    #[test]
    fn test_expr_from_vec() {
        let v: Vec<Expr> = vec![num(1), num(2), list!(3, 4).into()];
        assert_eq!(format!("{}", Expr::from(v)), "(1 2 (3 4))");
    }

    #[test]
    fn test_expr_from_bool() {
        assert_eq!(Expr::from(true), Expr::Bool(true, None));
        assert_eq!(Expr::from(false), Expr::Bool(false, None));
    }

    #[test]
    fn test_accessors_ignore_spans() {
        use crate::span::{Loc, Span};

        let span = Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)));
        assert_eq!(Expr::Num(1.5, span).as_num(), Some(1.5));
        assert_eq!(Expr::Bool(true, span).as_bool(), Some(true));
        assert_eq!(Expr::Str("hi".into(), span).as_str(), Some("hi"));
        assert_eq!(
            Expr::Sym(Symbol::intern("x"), span)
                .as_sym()
                .unwrap()
                .as_str(),
            "x"
        );
        assert!(Expr::from(1).as_str().is_none());

        let list = list!(1, 2);
        assert_eq!(Expr::List(list.clone(), span).as_list(), Some(&list));

        let foreign = Expr::foreign(Sample(7));
        assert_eq!(foreign.as_foreign::<Sample>().unwrap().0, 7);
        assert!(foreign.as_foreign::<String>().is_none());
        assert!(Expr::from(1).as_foreign::<Sample>().is_none());
    }

    #[test]
    fn test_from_usize_and_collect() {
        assert_eq!(Expr::from(3_usize), Expr::from(3));
        let list: List = [Expr::from(1), Expr::from(2)].into_iter().collect();
        assert_eq!(list.len(), 2);
        let expr: Expr = [Expr::from(1), Expr::from(2)].into_iter().collect();
        assert_eq!(expr, Expr::from(vec![Expr::from(1), Expr::from(2)]));
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(format!("{}", Expr::from(true)), "true");
        assert_eq!(format!("{}", Expr::from(false)), "false");
    }
}
