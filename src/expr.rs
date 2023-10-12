use crate::error::Loc;
use std::{collections::HashMap, fmt::Display, hash::Hash};

// used to allocate id for variables to differentiate
// between variables from different clauses
pub(crate) struct IdAlloc<T>(HashMap<T, u64>, u64);

impl<T: Eq + Hash> IdAlloc<T> {
    pub(crate) fn new(i: u64) -> Self {
        Self(HashMap::new(), i)
    }
    pub(crate) fn alloc(&mut self, s: T) -> u64 {
        *self.0.entry(s).or_insert_with(|| {
            self.1 += 1;
            self.1
        })
    }
    pub(crate) fn new_clause(&mut self) {
        self.0.clear();
    }
    pub(crate) fn get_next(&self) -> u64 {
        self.1 + 1
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Fun {
        name: String,
        args: Vec<Expr>,
        loc: Loc,
    },
    Var {
        name: String,
        id: u64,
        loc: Loc,
    },
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Var {
            name: String::default(),
            id: 0,
            loc: Loc::default(),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Var { id, .. }, Expr::Var { id: id2, .. }) => id == id2,
            (
                Expr::Fun { name, args, .. },
                Expr::Fun {
                    name: name2,
                    args: args2,
                    ..
                },
            ) => name == name2 && args == args2,
            _ => false,
        }
    }
}

impl Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::with_stacker(|| match self {
            Expr::Fun { name, args, .. } if args.is_empty() => write!(fmt, "{}", name),
            Expr::Fun { name, args, .. } => {
                write!(fmt, "{}(", name)?;
                let mut comma = false;
                for arg in args {
                    if comma {
                        write!(fmt, ", ")?;
                    }
                    write!(fmt, "{}", arg)?;
                    comma = true;
                }
                write!(fmt, ")")
            }
            Expr::Var { name, .. } => write!(fmt, "{}", name),
        })
    }
}
