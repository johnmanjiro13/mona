use std::{env, process};

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("Usage: 9cc <code>");
        process::exit(1);
    }

    let p = args.nth(1).unwrap();

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    let (n, mut p) = strtol(&p);
    println!("  mov rax, {}", n.unwrap());

    while let Some(c) = p.chars().next() {
        let s = p.split_off(1);

        if c == '+' {
            let (n, remaining) = strtol(&s);
            p = remaining;
            println!("  add rax, {}", n.unwrap());
            continue;
        }

        if c == '-' {
            let (n, remaining) = strtol(&s);
            p = remaining;
            println!("  sub rax, {}", n.unwrap());
            continue;
        }

        eprintln!("unexpected character: {}", p);
        process::exit(1);
    }

    println!("  ret");
}

pub fn strtol(s: &String) -> (Option<i64>, String) {
    if s.is_empty() {
        return (None, s.clone());
    }

    let mut pos = 0;
    let mut remaining = s.clone();
    let len = s.len();

    while len != pos {
        if !s.chars().nth(pos).unwrap().is_ascii_digit() {
            break;
        }
        pos += 1;
    }

    if len == pos {
        (Some(remaining.parse::<i64>().unwrap()), "".into())
    } else {
        let t: String = remaining.drain(..pos).collect();
        (Some(t.parse::<i64>().unwrap()), remaining)
    }
}
