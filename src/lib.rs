use error::Error;
use parser::{Def, Expr, IdAlloc};
use std::collections::HashMap;
use token::TokenTy;
use unify::{substitute_and_freshen, ApplyError};

mod error;
mod parser;
mod token;
mod unify;

pub type Rules = HashMap<String, Vec<Def>>;

pub struct Context {
    id: IdAlloc,
}

impl Context {
    pub fn new() -> Self {
        Self { id: IdAlloc::new() }
    }

    pub fn parse(&mut self, src: String) -> Result<Rules, Error> {
        let scanner = token::Scanner::new(&src);
        let mut parser = parser::Parser::new(scanner);
        let mut defs: HashMap<_, Vec<Def>> = HashMap::new();

        loop {
            self.id.new_clause();
            if let Some(mut def) = parser.parse_def(&mut self.id)? {
                def.rep.reverse();
                defs.entry(def.name.clone()).or_default().push(def);
            } else {
                break;
            }
        }
        Ok(defs)
    }

    pub fn parse_clause(&mut self, src: String) -> Result<Vec<Expr>, Error> {
        self.id.new_clause();
        let scanner = token::Scanner::new(&src);
        let mut parser = parser::Parser::new(scanner);
        let mut e = parser.parse_clause(&mut self.id)?;
        e.reverse();
        parser.sc.expect_token(TokenTy::Period)?;
        parser.sc.expect_token(TokenTy::Eof)?;

        Ok(e)
    }

    pub fn apply(&mut self, defs: &Rules, e: Vec<Expr>) {
        let mut qvars = HashMap::new();
        let mut order = Vec::new();
        vars(&mut qvars, &mut order, &e);
        match apply_internal(&mut self.id, defs, e.clone(), qvars) {
            Ok(sols) if !sols.is_empty() => {
                print_sols(sols, &order);
            }
            _ => println!("false."),
        }
    }
}
fn print_sols(mut sols: Vec<HashMap<&str, Expr>>, order: &[&str]) {
    let mut i = 0;
    while i < sols.len() {
        if sols[0..i].contains(&sols[i]) {
            sols.remove(i);
        }
        i += 1;
    }
    for sol in sols {
        if sol.is_empty() {
            println!("true.");
            break;
        }
        let mut c = false;
        for v in order {
            let e = &sol[v];
            if c {
                print!(", ");
            }
            print!("{} = {}", v, e);
            c = true;
        }
        println!(".")
    }
}

fn apply_internal<'a>(
    gen: &mut IdAlloc,
    defs: &Rules,
    mut e: Vec<Expr>,
    qvars: HashMap<&'a str, Expr>,
) -> Result<Vec<HashMap<&'a str, Expr>>, ApplyError> {
    with_stacker(|| {
        e.retain(
            |i| !matches!(i, Expr::Fun { name, args, .. } if name == "true" && args.is_empty()),
        );
        let curr_e = match e.pop() {
            Some(e) => e,
            _ => return Ok(vec![qvars]),
        };
        let f_defs = match &curr_e {
            Expr::Fun { name, .. } => {
                if let Some(x) = defs.get(name) {
                    x
                } else {
                    return Err(ApplyError::NoMatch);
                }
            }
            _ => return Err(ApplyError::Undef),
        };
        let v: Vec<_> = f_defs
            .iter()
            .filter_map(|x| x.apply(&curr_e).ok().map(|s| (&x.rep, s)))
            .collect();

        if v.is_empty() {
            Err(ApplyError::NoMatch)
        } else {
            gen.new_clause();

            let mut ret = Vec::new();
            for (rep, sub) in v {
                let qvars: HashMap<&str, Expr> = qvars
                    .iter()
                    .map(|(s, e)| (*s, substitute_and_freshen(gen, &sub, e)))
                    .collect();
                let e = e
                    .iter()
                    .chain(rep.iter())
                    .map(|e| substitute_and_freshen(gen, &sub, e))
                    .collect();
                ret.extend_from_slice(&apply_internal(gen, defs, e, qvars).unwrap_or_default());
            }
            Ok(ret)
        }
    })
}

fn vars<'a>(v: &mut HashMap<&'a str, Expr>, o: &mut Vec<&'a str>, e: &'a [Expr]) {
    for i in e {
        match i {
            Expr::Fun { args, .. } => vars(v, o, args),
            Expr::Var { name, .. } if !v.contains_key(&&name[..]) => {
                v.insert(name, i.clone());
                o.push(name)
            }
            _ => {}
        }
    }
}

pub fn with_stacker<R>(f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, f)
}
