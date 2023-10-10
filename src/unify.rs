use std::collections::HashMap;

use crate::{Def, Expr};

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
        (Expr::Var { id, .. }, _) => {
            if let Some(e2) = b.get(&id) {
                if *e2 != e {
                    Err(ApplyError::UnifyFail)
                } else {
                    Ok(())
                }
            } else {
                b.insert(*id, e);
                Ok(())
            }
        }
        (_, Expr::Var { id, .. }) => {
            if let Some(pat2) = b.get(&id) {
                if *pat2 != pat {
                    Err(ApplyError::UnifyFail)
                } else {
                    Ok(())
                }
            } else {
                b.insert(*id, pat);
                Ok(())
            }
        }

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

pub(crate) fn substitute(b: &HashMap<u64, &Expr>, rep: &Expr) -> Expr {
    match rep {
        Expr::Var { id, .. } => b.get(&id).copied().unwrap_or(&rep).clone(),
        Expr::Fun { name, args, loc } => Expr::Fun {
            name: name.clone(),
            loc: *loc,
            args: args.iter().map(|i| substitute(b, i)).collect(),
        },
    }
}
