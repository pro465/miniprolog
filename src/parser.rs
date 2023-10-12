use crate::{
    error::{Error, Loc},
    expr::{Expr, IdAlloc},
    token::{Scanner, TokenTy},
};

#[derive(Clone, Debug)]
pub struct Def {
    pub name: String,
    pub loc: Loc,
    pub(crate) pat: Expr,
    pub(crate) rep: Vec<Expr>,
}

pub struct Parser<'a> {
    pub(crate) sc: Scanner<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(sc: Scanner<'a>) -> Self {
        Self { sc }
    }

    pub(crate) fn parse_def(&mut self, id: &mut IdAlloc<String>) -> Result<Option<Def>, Error> {
        if self.sc.peek()?.ty() == TokenTy::Eof {
            return Ok(None);
        }
        let (name, loc, pat) = self.parse_expr(id)?;
        let rep = if self.sc.is_token(TokenTy::Pen)? {
            self.parse_clause(id)?
        } else {
            Vec::new()
        };
        self.sc.expect_token(TokenTy::Period)?;

        Ok(Some(Def {
            name,
            loc,
            pat,
            rep,
        }))
    }

    pub(crate) fn parse_clause(&mut self, id: &mut IdAlloc<String>) -> Result<Vec<Expr>, Error> {
        let mut v = Vec::new();
        loop {
            v.push(self.parse_expr(id)?.2);
            if !self.sc.is_token(TokenTy::Comma)? {
                break Ok(v);
            }
        }
    }

    fn parse_expr(&mut self, id: &mut IdAlloc<String>) -> Result<(String, Loc, Expr), Error> {
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
            let id = id.alloc(name.clone());
            Expr::Var { name, id, loc }
        };

        Ok((name, loc, res))
    }
}
