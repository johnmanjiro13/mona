use core::panic;
use std::{env, process, sync::Mutex};

use lazy_static::lazy_static;

// Tokenizer
#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    Num,
    Plus,
    Minus,
}

impl From<char> for TokenType {
    fn from(c: char) -> Self {
        match c {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            e => panic!("unknown token type: {}", e),
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Num
    }
}

#[derive(Default, Debug)]
struct Token {
    ty: TokenType,
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
                ty: TokenType::from(c),
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
                ty: TokenType::Num,
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

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Num,
    Plus,
    Minus,
}

impl From<TokenType> for NodeType {
    fn from(t: TokenType) -> Self {
        match t {
            TokenType::Num => NodeType::Num,
            TokenType::Plus => NodeType::Plus,
            TokenType::Minus => NodeType::Minus,
        }
    }
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Num
    }
}

#[derive(Default, Clone)]
struct Node {
    ty: NodeType,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: i32,
}

impl Node {
    fn new(op: NodeType, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Self {
            ty: op,
            lhs: Some(lhs),
            rhs: Some(rhs),
            ..Default::default()
        }
    }

    fn new_num(val: i32) -> Self {
        Self {
            ty: NodeType::Num,
            val,
            ..Default::default()
        }
    }

    fn number(tokens: &Vec<Token>, pos: usize) -> Self {
        let t = &tokens[pos];
        if t.ty != TokenType::Num {
            panic!("number expected, but got {}", t.input);
        }
        Self::new_num(t.val)
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

            let op = tokens[pos].ty.clone();
            if op != TokenType::Plus && op != TokenType::Minus {
                break;
            }
            pos += 1;
            lhs = Self::new(
                NodeType::from(op),
                Box::new(lhs),
                Box::new(Self::number(&tokens, pos)),
            );
            pos += 1;
        }
        if tokens.len() != pos {
            panic!("stray token: {}", tokens[pos].input);
        }
        lhs
    }
}

// Intermediate representation

#[derive(Debug, Clone)]
enum IRType {
    IMM,
    MOV,
    RETURN,
    KILL,
    NOP,
    ADD,
    SUB,
}

impl From<NodeType> for IRType {
    fn from(t: NodeType) -> Self {
        match t {
            NodeType::Plus => IRType::ADD,
            NodeType::Minus => IRType::SUB,
            e => panic!("cannot convert: {:?}", e),
        }
    }
}

#[derive(Clone)]
struct IR {
    op: IRType,
    lhs: usize,
    rhs: usize,
}

impl IR {
    fn new(op: IRType, lhs: usize, rhs: usize) -> Self {
        Self { op, lhs, rhs }
    }
}

fn gen_ir_sub(mut v: Vec<IR>, node: Node) -> (usize, Vec<IR>) {
    if node.ty == NodeType::Num {
        let r = v.len();
        v.push(IR::new(IRType::IMM, r, node.val as usize));
        return (r, v);
    }

    assert!(node.ty == NodeType::Plus || node.ty == NodeType::Minus);

    let (lhs, ins) = gen_ir_sub(v, *node.lhs.unwrap());
    let (rhs, mut ins) = gen_ir_sub(ins, *node.rhs.unwrap());

    ins.push(IR::new(IRType::from(node.ty), lhs, rhs));
    ins.push(IR::new(IRType::KILL, rhs, 0));
    (lhs, ins)
}

fn gen_ir(node: Node) -> Vec<IR> {
    let (r, mut ins) = gen_ir_sub(vec![], node);
    ins.push(IR::new(IRType::RETURN, r, 0));
    ins
}

// Register allocator

const REGS_N: usize = 8;
const REGS: [&str; REGS_N] = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];

lazy_static! {
    static ref USED: Mutex<[bool; REGS_N]> = Mutex::new([false; REGS_N]);
    static ref REG_MAP: Mutex<Vec<Option<usize>>> = Mutex::new(vec![]);
}

fn used_get(i: usize) -> bool {
    USED.lock().unwrap()[i]
}

fn used_set(i: usize, val: bool) {
    USED.lock().unwrap()[i] = val;
}

fn reg_map_get(i: usize) -> Option<usize> {
    REG_MAP.lock().unwrap().get(i).cloned().unwrap()
}

fn reg_map_set(i: usize, val: usize) {
    REG_MAP.lock().unwrap()[i] = Some(val);
}

fn alloc(ir_reg: usize) -> usize {
    if let Some(r) = reg_map_get(ir_reg) {
        assert!(used_get(r));
        return r;
    }

    for i in 0..REGS_N {
        if used_get(i) {
            continue;
        }
        used_set(i, true);
        reg_map_set(ir_reg, i);
        return i;
    }
    panic!("register exhauseted");
}

fn kill(r: usize) {
    assert!(used_get(r));
    used_set(r, false);
}

fn alloc_regs(irv: Vec<IR>) -> Vec<IR> {
    use IRType::*;
    let mut new: Vec<IR> = vec![];
    let irv_len = irv.len();

    *REG_MAP.lock().unwrap() = vec![None; irv_len];

    for i in 0..irv_len {
        let mut ir = irv[i].clone();
        match ir.op {
            IMM => ir.lhs = alloc(ir.lhs),
            RETURN => kill(reg_map_get(ir.lhs).unwrap()),
            KILL => {
                kill(reg_map_get(ir.lhs).unwrap());
                ir.op = IRType::NOP;
            }
            ADD | SUB | MOV => {
                ir.lhs = alloc(ir.lhs);
                ir.rhs = alloc(ir.rhs);
            }
            op => panic!("unknown operator: {:?}", op),
        }
        new.push(ir);
    }
    new
}

// Code generator
fn gen_x86(irv: Vec<IR>) {
    use IRType::*;
    for ir in irv.clone() {
        match ir.op {
            IMM => println!("  mov {}, {}", REGS[ir.lhs], ir.rhs),
            MOV => println!("  mov {}, {}", REGS[ir.lhs], REGS[ir.rhs]),
            RETURN => {
                println!("  mov rax, {}", REGS[ir.lhs]);
                println!("  ret");
            }
            ADD => println!("  add {}, {}", REGS[ir.lhs], REGS[ir.rhs]),
            SUB => println!("  sub {}, {}", REGS[ir.lhs], REGS[ir.rhs]),
            NOP | KILL => (),
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
    let irv = gen_ir(node);
    let irv_allocated = alloc_regs(irv);

    // Prologue
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen_x86(irv_allocated);
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
