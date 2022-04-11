use std::env;

use mona::codegen::gen_x86;
use mona::ir::{dump_ir, gen_ir};
use mona::parse::Node;
use mona::regalloc::alloc_regs;
use mona::token::tokenize;

fn main() {
    let mut args = env::args();
    let input: String;
    let mut dump_ir1 = false;
    let mut dump_ir2 = false;

    if args.len() == 3 {
        match args.nth(1).unwrap().as_str() {
            "-dump-ir1" => {
                dump_ir1 = true;
                input = args.next().unwrap();
            }
            "-dump-ir2" => {
                dump_ir2 = true;
                input = args.next().unwrap();
            }
            _ => {
                panic!("invalid flag");
            }
        }
    } else {
        if args.len() != 2 {
            eprintln!("Usage: 9cc <code>");
            return;
        }
        input = args.nth(1).unwrap();
    }

    // Tokenize and parse.
    let tokens = tokenize(input);
    let node = Node::parse(&tokens);

    let mut irv = gen_ir(node);

    if dump_ir1 {
        dump_ir(&irv);
    }

    alloc_regs(&mut irv);

    if dump_ir2 {
        dump_ir(&irv);
    }

    // Prologue
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen_x86(irv);
}
