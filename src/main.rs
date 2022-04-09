use core::panic;
use std::{env, process, sync::Mutex};

use lazy_static::lazy_static;
use num::traits::FromPrimitive;

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

#[derive(Default, Clone)]
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
}

// Intermediate representation

enum IRType {
    IMM,
    MOV,
    RETURN,
    KILL,
    NOP,
}

impl FromPrimitive for IRType {
    fn from_i64(n: i64) -> Option<Self> {
        use IRType::*;
        match n {
            0 => Some(IMM),
            1 => Some(MOV),
            2 => Some(RETURN),
            3 => Some(KILL),
            4 => Some(NOP),
            _ => None,
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        use IRType::*;
        match n {
            0 => Some(IMM),
            1 => Some(MOV),
            2 => Some(RETURN),
            3 => Some(KILL),
            4 => Some(NOP),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct IR {
    op: i32,
    lhs: i32,
    rhs: i32,
}

impl IR {
    fn new(op: i32, lhs: i32, rhs: i32) -> Self {
        Self { op, lhs, rhs }
    }
}

lazy_static! {
    static ref INS: Mutex<Vec<IR>> = Mutex::new(vec![]);
}

static mut REGNO: usize = 0;

fn gen_ir_sub(node: Node) -> i32 {
    if node.ty == NodeType::Num as i32 {
        let r: i32;
        unsafe {
            r = REGNO as i32;
            REGNO += 1;
        }
        INS.lock()
            .unwrap()
            .push(IR::new(IRType::IMM as i32, r, node.val));
        return r;
    }

    assert!(node.ty == '+' as i32 || node.ty == '-' as i32);

    let lhs = gen_ir_sub(*node.lhs.unwrap());
    let rhs = gen_ir_sub(*node.rhs.unwrap());

    INS.lock().unwrap().push(IR::new(node.ty, lhs, rhs));
    INS.lock()
        .unwrap()
        .push(IR::new(IRType::KILL as i32, rhs, 0));
    lhs
}

fn gen_ir(node: Node) {
    let r = gen_ir_sub(node);
    INS.lock()
        .unwrap()
        .push(IR::new(IRType::RETURN as i32, r, 0));
}

// Register allocator
const REGS: [&str; 8] = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];

lazy_static! {
    static ref USED: Mutex<[bool; 8]> = Mutex::new([false; 8]);
    static ref REG_MAP: Mutex<[i32; 1000]> = Mutex::new([-1; 1000]);
}

fn used_get(i: usize) -> bool {
    USED.lock().unwrap()[i]
}

fn used_set(i: usize, val: bool) {
    USED.lock().unwrap()[i] = val;
}

fn reg_map_get(i: usize) -> i32 {
    REG_MAP.lock().unwrap()[i]
}

fn reg_map_set(i: usize, val: i32) {
    REG_MAP.lock().unwrap()[i] = val;
}

fn alloc(ir_reg: usize) -> i32 {
    if reg_map_get(ir_reg) != -1 {
        let r = reg_map_get(ir_reg);
        assert!(used_get(r as usize));
        return r;
    }

    for i in 0..REGS.len() {
        if used_get(i) {
            continue;
        }
        used_set(i, true);
        reg_map_set(ir_reg, i as i32);
        return i as i32;
    }
    panic!("register exhauseted");
}

fn kill(r: usize) {
    assert!(used_get(r));
    used_set(r, false);
}

fn alloc_regs() {
    use IRType::*;
    let mut ins_allocated: Vec<IR> = vec![];

    for mut ir in INS.lock().unwrap().clone() {
        match IRType::from_i32(ir.op) {
            Some(IMM) => ir.lhs = alloc(ir.lhs as usize),
            Some(MOV) => {
                ir.lhs = alloc(ir.lhs as usize);
                ir.rhs = alloc(ir.rhs as usize);
            }
            Some(RETURN) => kill(reg_map_get(ir.lhs as usize) as usize),
            Some(KILL) => {
                kill(reg_map_get(ir.lhs as usize) as usize);
                ir.op = IRType::NOP as i32;
            }
            None => match ir.op as u8 as char {
                '+' | '-' => {
                    ir.lhs = alloc(ir.lhs as usize);
                    ir.rhs = alloc(ir.rhs as usize);
                }
                _ => panic!("unknown operator"),
            },
            _ => panic!("unknown operator"),
        }
        ins_allocated.push(ir);
    }

    for i in 0..ins_allocated.len() {
        INS.lock().unwrap()[i] = ins_allocated[i].clone();
    }
}

// Code generator
fn gen_x86() {
    use IRType::*;
    for ir in INS.lock().unwrap().clone() {
        match IRType::from_i32(ir.op) {
            Some(IMM) => println!("  mov {}, {}", REGS[ir.lhs as usize], ir.rhs),
            Some(MOV) => println!("  mov {}, {}", REGS[ir.lhs as usize], REGS[ir.rhs as usize]),
            Some(RETURN) => {
                println!("  mov rax, {}", REGS[ir.lhs as usize]);
                println!("  ret");
            }
            Some(NOP) | Some(KILL) => (),
            None => match ir.op as u8 as char {
                '+' => println!("  add {}, {}", REGS[ir.lhs as usize], REGS[ir.rhs as usize]),
                '-' => println!("  sub {}, {}", REGS[ir.lhs as usize], REGS[ir.rhs as usize]),
                _ => panic!("unknown operator"),
            },
        }
    }
}

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("Usage: 9cc <code>");
        process::exit(1);
    }

    // Tokenize and parse.
    let tokens = tokenize(args.nth(1).unwrap());
    let node = Node::expr(tokens);
    gen_ir(node.clone());
    alloc_regs();

    // Prologue
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen_x86();

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
