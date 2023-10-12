use std::collections::HashMap;

use crate::{
    expr::{Expr, IdAlloc},
    Def,
};

#[derive(Debug)]
pub(crate) enum ApplyError {
    UnifyFail,
    Undef,
    NoMatch,
}

impl Def {
    pub(crate) fn apply(&self, e: &Expr) -> Result<HashMap<u64, Expr>, ApplyError> {
        let mut bindings = HashMap::new();
        unify(&mut bindings, &self.pat, e)?;
        Ok(bindings.into_iter().map(|(k, v)| (k, v.clone())).collect())
    }
}

// try to unify 2 expressions
fn unify<'a>(b: &mut HashMap<u64, &'a Expr>, pat: &'a Expr, e: &'a Expr) -> Result<(), ApplyError> {
    match (pat, e) {
        (Expr::Var { id, .. }, Expr::Var { id: id2, .. }) if b.contains_key(id2) => {
            solve(b, *id, b[id2])
        }
        (Expr::Var { id, .. }, _) => solve(b, *id, e),

        (_, Expr::Var { id, .. }) => solve(b, *id, pat),

        (
            Expr::Fun { name, args, .. },
            Expr::Fun {
                name: name2,
                args: args2,
                ..
            },
        ) if name == name2 => {
            for (arg1, arg2) in args.iter().zip(args2.iter()) {
                unify(b, arg1, arg2)?;
            }
            Ok(())
        }
        _ => Err(ApplyError::UnifyFail),
    }
}

// replace all the variables by their replacement given in bindings
// and freshen up the younglings
pub(crate) fn substitute_and_freshen(
    gen: &mut IdAlloc<u64>,
    b: &HashMap<u64, Expr>,
    rep: &Expr,
) -> Expr {
    match rep {
        Expr::Var { id, .. } => freshen(b.get(&id).unwrap_or(rep), gen),
        Expr::Fun { name, args, loc } => Expr::Fun {
            name: name.clone(),
            loc: *loc,
            args: args
                .iter()
                .map(|i| substitute_and_freshen(gen, b, i))
                .collect(),
        },
    }
}

// freshens up an expression by giving it coffee
// (or more accurately, giving the variables new ids)
fn freshen(e: &Expr, gen: &mut IdAlloc<u64>) -> Expr {
    match e {
        Expr::Var { name, id, loc } => Expr::Var {
            name: name.clone(),
            id: gen.alloc(*id),
            loc: *loc,
        },
        Expr::Fun { name, args, loc } => Expr::Fun {
            name: name.clone(),
            args: args.iter().map(|i| freshen(i, gen)).collect(),
            loc: *loc,
        },
    }
}

// solves the problem of unifying one variable that has already been unified
// for example, the second X in
//     eq(X, X).
// when we need to unify an expression, such as `eq(Z, Y)`
// by the time we get to unify X and Y, we already have the binding x -> Z in our map,
// so we cannot just insert X -> Z.
//
// how does this function do it?
// =============================
// this function just tries different combinations and sees if they work.
fn solve<'a>(b: &mut HashMap<u64, &'a Expr>, id: u64, e: &'a Expr) -> Result<(), ApplyError> {
    if let Some(e2) = b.get(&id).copied() {
        let res = if e == e2 {
            Ok(())
        } else if let Expr::Var { id, .. } = e {
            solve(b, *id, e2)
        } else {
            Err(ApplyError::UnifyFail)
        };
        if res.is_ok() {
            return res;
        }
        if let Expr::Var { id: id2, .. } = e2 {
            let res2 = solve(b, *id2, e);
            if res2.is_ok() {
                *b.get_mut(&id).unwrap() = e;
            }
            res2
        } else {
            Err(ApplyError::UnifyFail)
        }
    } else {
        b.insert(id, e);
        Ok(())
    }
}
