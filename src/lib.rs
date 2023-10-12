use error::Error;
use expr::{Expr, IdAlloc};
use parser::Def;
use std::collections::HashMap;
use token::TokenTy;
use unify::{substitute_and_freshen, ApplyError};

mod error;
mod expr;
mod parser;
mod token;
mod unify;

pub type Rules = HashMap<String, Vec<Def>>;

pub struct Context {
    id: IdAlloc<String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            id: IdAlloc::new(0),
        }
    }

    // parse the "program" from the file
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

    // parse the input from the REPL
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

    // run the program on the input
    pub fn apply(&mut self, defs: &Rules, e: Vec<Expr>) {
        let mut qvars = HashMap::new();
        let mut order = Vec::new();
        vars(&mut qvars, &mut order, &e);
        match apply_internal(self.id.get_next(), defs, e.clone(), qvars) {
            Ok(sols) if !sols.is_empty() => {
                print_sols(sols, &order);
            }
            _ => println!("No."),
        }
    }
}

// print the solution(s) if any
// sols is just the list of solutions
// for example,
//    X = state, Y = run.
//    X = state, Y = walk.
//  would be (roughly) represented as
//    [{X: state, Y: run}, {X: state, Y: walk}]
fn print_sols(mut sols: Vec<HashMap<&str, Expr>>, order: &[&str]) {
    let mut i = 0;
    // TODO: fix time complexity
    while i < sols.len() {
        if sols[0..i].contains(&sols[i]) {
            sols.remove(i);
        }
        i += 1;
    }
    for mut sol in sols {
        let mut c = false;
        for v in order {
            let e = &sol[v];
            // ignore things like Z = Z
            match e {
                Expr::Var { name, .. } if name == v => {
                    sol.remove(v);
                    continue;
                }
                _ => {}
            }
            if c {
                print!(", ");
            }
            print!("{} = {}", v, e);
            c = true;
        }
        // when the query has no variables, the binding set would be empty.
        // then it simply is a yes or no question.
        if sol.is_empty() {
            print!("Yes");
        }
        println!(".")
    }
}

// recursive implementation of the selection + SLD algorithm + backtracing
fn apply_internal<'a>(
    gen: u64,
    defs: &Rules,
    mut e: Vec<Expr>,
    qvars: HashMap<&'a str, Expr>,
) -> Result<Vec<HashMap<&'a str, Expr>>, ApplyError> {
    with_stacker(|| {
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
            let mut alloc = IdAlloc::new(gen);
            let mut ret = Vec::new();
            for (rep, sub) in v {
                // apply the same substitution that is applied to the goal in the SLD algorithm.
                // (see below)
                let qvars: HashMap<&str, Expr> = qvars
                    .iter()
                    .map(|(s, e)| (*s, substitute_and_freshen(&mut alloc, &sub, e)))
                    .collect();
                let e = e
                    .iter()
                    .chain(rep.iter())
                    .map(|e| substitute_and_freshen(&mut alloc, &sub, e))
                    .collect();
                ret.extend_from_slice(&apply_internal(gen, defs, e, qvars).unwrap_or_default());
            }
            Ok(ret)
        }
    })
}

// initializes the solution binding set (the set which holds the bindings used in `print_sols`)
// to maps between variables and themselves. each of them looks like A = A.
// these are then applied the same substitution that is applied to the goal in the SLD algorithm.
// ensuring that the final result is the map from the variables to those values that result in the
// empty clause.
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

// avoid stack overflows
pub fn with_stacker<R>(f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, f)
}
