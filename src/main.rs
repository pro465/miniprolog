use io::Write;
use miniprolog::{Context, Rules};
use std::{fs, io};

fn main() {
    let mut args = std::env::args();
    let mut ctx = Context::new();

    let rules = ctx.parse(
        fs::read_to_string(
            fs::canonicalize(args.nth(1).unwrap_or_else(|| help()))
                .expect("could not canonicalize argument"),
        )
        .expect("could not read file"),
    );
    let rules = rules.unwrap_or_else(|e| {
        e.report();
        std::process::exit(-1);
    });
    //dbg!(&rules);
    repl(&rules, &mut ctx);
}

fn repl(rules: &Rules, ctx: &mut Context) {
    println!("welcome to miniprolog v0.1.0!\ninput `q`, `quit`, or `exit` for exiting the REPL");

    let mut line = String::new();

    let mut prompt = |s| {
        print!("{s} ");
        io::stdout().flush().unwrap();
        line.clear();
        std::io::stdin()
            .read_line(&mut line)
            .expect("could not read input");
        let line = line.trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    };

    while let Some(mut line) = prompt("?-") {
        if is_quit(&line) {
            break;
        }
        if !line.contains('.') {
            loop {
                let t = match prompt("..") {
                    Some(x) if !is_quit(&x) => x,
                    _ => break,
                };
                line.push_str(&t);
                if t.contains('.') {
                    break;
                }
            }
        }

        let expr = ctx.parse_clause(line);
        let expr = match expr {
            Ok(x) => x,
            Err(e) => {
                e.report();
                continue;
            }
        };

        let mut sols_printer = ctx.apply(&rules, &expr);
        let mut line = String::new();

        'outer: while sols_printer.print_next_sol() {
            loop {
                line.clear();
                std::io::stdout().flush().unwrap();
                std::io::stdin()
                    .read_line(&mut line)
                    .expect("could not read input");

                let trimmed = line.trim();
                if trimmed == "." {
                    break 'outer;
                } else if trimmed == ";" {
                    break;
                } else {
                    println!("Input `;` to see next solution, or `.` to not.");
                }
            }
        }
    }
}

fn help() -> ! {
    println!(
        "usage: {} <filename>",
        std::env::current_exe()
            .unwrap_or_else(|_| "miniprolog".into())
            .display()
    );
    std::process::exit(-1);
}

fn is_quit(s: &str) -> bool {
    ["quit", "exit", "q"].contains(&s)
}
