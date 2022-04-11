use std::{env, process};

use mona::codegen::gen_x86;
use mona::ir::gen_ir;
use mona::parse::Node;
use mona::regalloc::alloc_regs;
use mona::token::tokenize;

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("Usage: 9cc <code>");
        process::exit(1);
    }

    // Tokenize and parse.
    let tokens = tokenize(args.nth(1).unwrap());
    let node = Node::parse(&tokens);
    let irv = gen_ir(node);
    let irv_allocated = alloc_regs(irv);

    // Prologue
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen_x86(irv_allocated);
}
