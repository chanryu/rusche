use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

/// An interned symbol name.
///
/// Equal symbols share one `Rc<str>`, so [`PartialEq`] and [`Hash`] use pointer identity
/// and cloning a symbol is a refcount bump.
#[derive(Clone, Debug)]
pub struct Symbol(Rc<str>);

impl Symbol {
    /// Interns `name`, returning a symbol that pointer-compares equal to every other
    /// intern of the same text on this thread.
    pub fn intern(name: impl AsRef<str>) -> Self {
        thread_local! {
            static INTERNER: RefCell<HashMap<Box<str>, Rc<str>>> =
                RefCell::new(HashMap::new());
        }

        let name = name.as_ref();
        INTERNER.with(|interner| {
            let mut map = interner.borrow_mut();
            if let Some(rc) = map.get(name) {
                return Symbol(rc.clone());
            }
            let rc: Rc<str> = Rc::from(name);
            map.insert(Box::from(name), rc.clone());
            Symbol(rc)
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_rc(&self) -> &Rc<str> {
        &self.0
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol::intern(value)
    }
}

impl From<String> for Symbol {
    fn from(value: String) -> Self {
        Symbol::intern(value)
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl PartialEq<&str> for Symbol {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_shares_identity() {
        let a = Symbol::intern("foo");
        let b = Symbol::intern("foo");
        let c = Symbol::intern("bar");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(Rc::ptr_eq(a.as_rc(), b.as_rc()));
        assert_eq!(&*a, "foo");
    }
}
