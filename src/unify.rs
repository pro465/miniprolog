use std::collections::HashMap;

use crate::{parser::IdAlloc, Def, Expr};

#[derive(Debug)]
pub(crate) enum ApplyError {
    UnifyFail,
    Undef,
    NoMatch,
}

impl Def {
    pub(crate) fn apply<'a>(&'a self, e: &'a Expr) -> Result<HashMap<u64, &'a Expr>, ApplyError> {
        let mut bindings = HashMap::new();
        unify(&mut bindings, &self.pat, e)?;
        Ok(bindings)
    }
}

fn unify<'a>(b: &mut HashMap<u64, &'a Expr>, pat: &'a Expr, e: &'a Expr) -> Result<(), ApplyError> {
    match (pat, e) {
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

pub(crate) fn substitute_and_freshen(
    gen: &mut IdAlloc,
    b: &HashMap<u64, &Expr>,
    rep: &Expr,
) -> Expr {
    match rep {
        Expr::Var { id, .. } => freshen(b.get(&id).copied().unwrap_or(&rep), gen),
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
fn freshen(e: &Expr, gen: &mut IdAlloc) -> Expr {
    match e {
        Expr::Var { name, id, loc } => Expr::Var {
            name: name.clone(),
            id: id + gen.alloc(name),
            loc: *loc,
        },
        Expr::Fun { name, args, loc } => Expr::Fun {
            name: name.clone(),
            args: args.iter().map(|i| freshen(i, gen)).collect(),
            loc: *loc,
        },
    }
}

// X - Y
// |
// X - Z

fn solve<'a>(b: &mut HashMap<u64, &'a Expr>, id: u64, e: &'a Expr) -> Result<(), ApplyError> {
    if let Some(e2) = b.get(&id).copied() {
        if e2 != e {
            let res = if let Expr::Var { id, .. } = e {
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
                    b.remove(&id);
                    b.insert(id, e);
                }
                res2
            } else {
                Err(ApplyError::UnifyFail)
            }
        } else {
            Ok(())
        }
    } else {
        b.insert(id, e);
        Ok(())
    }
}
