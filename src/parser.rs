use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    error::{Error, Loc},
    token::{Scanner, TokenTy},
};

#[derive(Clone, Debug)]
pub struct Def {
    pub name: String,
    pub loc: Loc,
    pub(crate) pat: Expr,
    pub(crate) rep: Vec<Expr>,
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

impl Expr {
    pub(crate) fn loc(&self) -> Loc {
        match self {
            Expr::Var { loc, .. } | Expr::Fun { loc, .. } => *loc,
        }
    }
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

pub(crate) struct IdAlloc(HashMap<String, u64>, u64);

impl IdAlloc {
    pub(crate) fn new() -> Self {
        Self(HashMap::new(), 0)
    }
    pub(crate) fn alloc(&mut self, s: &str) -> u64 {
        *self.0.entry(s.to_string()).or_insert_with(|| {
            self.1 += 1;
            self.1
        })
    }
    pub(crate) fn new_clause(&mut self) {
        self.0.clear();
    }
}

pub struct Parser<'a> {
    pub(crate) sc: Scanner<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(sc: Scanner<'a>) -> Self {
        Self { sc }
    }
    pub(crate) fn parse_def(&mut self, id: &mut IdAlloc) -> Result<Option<Def>, Error> {
        if self.sc.peek()?.ty() == TokenTy::Eof {
            return Ok(None);
        }
        let (name, loc, pat) = self.parse_expr(id)?;
        self.sc.expect_token(TokenTy::Pen)?;
        let rep = self.parse_clause(id)?;
        self.sc.expect_token(TokenTy::Period)?;

        Ok(Some(Def {
            name,
            loc,
            pat,
            rep,
        }))
    }

    pub(crate) fn parse_clause(&mut self, id: &mut IdAlloc) -> Result<Vec<Expr>, Error> {
        let mut v = Vec::new();
        loop {
            v.push(self.parse_expr(id)?.2);
            if !self.sc.is_token(TokenTy::Comma)? {
                break Ok(v);
            }
        }
    }

    fn parse_expr(&mut self, id: &mut IdAlloc) -> Result<(String, Loc, Expr), Error> {
        let (loc, name) = self.sc.expect_identifier()?;

        let res = if name.chars().next().unwrap().is_lowercase() {
            let name = name.clone();
            let args = if self.sc.is_token(TokenTy::Lparen)? {
                let args = self.parse_clause(id)?;
                self.sc.expect_token(TokenTy::Rparen)?;
                args
            } else {
                Vec::new()
            };
            Expr::Fun { name, args, loc }
        } else {
            let name = name.clone();
            let id = id.alloc(&name);
            Expr::Var { name, id, loc }
        };

        Ok((name, loc, res))
    }
}
