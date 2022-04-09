use std::{env, process, thread::panicking};

const REGS: [&str; 8] = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
static mut cur: usize = 0;

// Tokenizer
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

// Recursive-descendent parser

enum NodeType {
    Num,
}

#[derive(Default)]
struct Node {
    ty: i32,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: i32,
}

impl Node {
    fn new(op: i32, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            ty: op,
            lhs: Some(lhs),
            rhs: Some(rhs),
            ..Default::default()
        }
    }

    fn new_num(val: i32) -> Self {
        Self {
            ty: NodeType::Num as i32,
            val,
            ..Default::default()
        }
    }

    fn number(tokens: &Vec<Token>, pos: usize) -> Self {
        if tokens[pos].ty == TokenType::Num as i32 {
            let val = tokens[pos].val;
            return Self::new_num(val);
        }
        panic!("number expected, but got {}", tokens[pos].input);
    }

    pub fn expr(tokens: Vec<Token>) -> Self {
        let mut pos = 0;
        let mut lhs = Self::number(&tokens, pos);
        pos += 1;

        if tokens.len() == pos {
            return lhs;
        }

        loop {
            if tokens.len() == pos {
                break;
            }

            let op = tokens[pos].ty;
            if op != '+' as i32 && op != '-' as i32 {
                break;
            }
            pos += 1;
            lhs = Self::new(op, Box::new(lhs), Box::new(Self::number(&tokens, pos)));
            pos += 1;
        }
        if tokens.len() != pos {
            panic!("stray token: {}", tokens[pos].input);
        }
        lhs
    }

    // Code generator
    fn gen(self) -> String {
        if self.ty == NodeType::Num as i32 {
            let reg: &str;
            unsafe {
                if cur > REGS.len() {
                    panic!("register exhausted");
                }
                reg = REGS[cur];
                cur += 1;
            }
            println!("  mov {}, {}", reg, self.val);
            return reg.into();
        }

        let dst = self.lhs.unwrap().gen();
        let src = self.rhs.unwrap().gen();
        match self.ty as u8 as char {
            '+' => {
                println!("  add {}, {}", dst, src);
                dst
            }
            '-' => {
                println!("  sub {}, {}", dst, src);
                dst
            }
            _ => panic!("unknown operator"),
        }
    }
}

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("Usage: 9cc <code>");
        process::exit(1);
    }

    let tokens = tokenize(args.nth(1).unwrap());
    let node = Node::expr(tokens);

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    println!(" mov rax, {}", node.gen());

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
