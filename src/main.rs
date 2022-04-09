use std::{env, process};

enum TokenType {
    Num,
}

#[derive(Default, Debug)]
struct Token {
    ty: i32,
    val: i32,
    input: String,
}

fn tokenize(mut p: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let org = p.clone();
    while let Some(c) = p.chars().next() {
        if c.is_whitespace() {
            p = p.split_off(1);
            continue;
        }

        if c == '+' || c == '-' {
            let token = Token {
                ty: c as i32,
                input: org.clone(),
                ..Default::default()
            };
            p = p.split_off(1);
            tokens.push(token);
            continue;
        }

        if c.is_ascii_digit() {
            let (n, remaining) = strtol(&p);
            p = remaining;
            let token = Token {
                ty: TokenType::Num as i32,
                input: org.clone(),
                val: n.unwrap() as i32,
            };
            tokens.push(token);
            continue;
        }

        eprintln!("cannot tokenize: {}", p);
        process::exit(1);
    }
    tokens
}

fn fail(token: &Token) {
    eprintln!("unexpected char: {:?}", token);
    process::exit(1);
}

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("Usage: 9cc <code>");
        process::exit(1);
    }

    let tokens = tokenize(args.nth(1).unwrap());

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    if tokens[0].ty != TokenType::Num as i32 {
        fail(&tokens[0]);
    }
    println!(" mov rax, {}", tokens[0].val);

    let mut i = 1;
    while i != tokens.len() {
        if tokens[i].ty == '+' as i32 {
            i += 1;
            if tokens[i].ty != TokenType::Num as i32 {
                fail(&tokens[i]);
            }
            println!("  add rax, {}", tokens[i].val);
            i += 1;
            continue;
        }

        if tokens[i].ty == '-' as i32 {
            i += 1;
            if tokens[i].ty != TokenType::Num as i32 {
                fail(&tokens[i]);
            }
            println!("  sub rax, {}", tokens[i].val);
            i += 1;
            continue;
        }

        fail(&tokens[i]);
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
