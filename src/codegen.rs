use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::ir::{IROp, IR};
use crate::REGS;

lazy_static! {
    static ref N: Mutex<usize> = Mutex::new(0);
}

pub fn gen_x86(irv: Vec<IR>) {
    use IROp::*;
    let ret = ".Lend";

    println!("  push rbp");
    println!("  mov rbp, rsp");

    for ir in irv {
        let lhs = ir.lhs.unwrap();
        match ir.op {
            Imm => println!("  mov {}, {}", REGS[lhs], ir.rhs.unwrap()),
            Mov => println!("  mov {}, {}", REGS[lhs], REGS[ir.rhs.unwrap()]),
            Return => {
                println!("  mov rax, {}", REGS[lhs]);
                println!("  jmp {}", ret);
            }
            Label => println!(".L{}:", lhs),
            Unless => {
                println!("  cmp {}, 0", REGS[lhs]);
                println!("  je .L{}", ir.rhs.unwrap());
            }
            Alloca => {
                if ir.rhs.is_some() {
                    println!("  sub rsp, {}", ir.rhs.unwrap());
                }
                println!("  mov {}, rsp", REGS[lhs]);
            }
            Load => println!("  mov {}, [{}]", REGS[lhs], REGS[ir.rhs.unwrap()]),
            Store => println!("  mov [{}], {}", REGS[lhs], REGS[ir.rhs.unwrap()]),
            Add => println!(" add {}, {}", REGS[lhs], REGS[ir.rhs.unwrap()]),
            AddImm => println!("  add {}, {}", REGS[lhs], ir.rhs.unwrap()),
            Sub => println!("  sub {}, {}", REGS[lhs], REGS[ir.rhs.unwrap()]),
            Mul => {
                println!("  mov rax, {}", REGS[ir.rhs.unwrap()]);
                println!("  mul {}", REGS[lhs]);
                println!("  mov {}, rax", REGS[lhs]);
            }
            Div => {
                println!("  mov rax, {}", REGS[lhs]);
                println!("  cqo");
                println!("  div {}", REGS[ir.rhs.unwrap()]);
                println!("  mov {}, rax", REGS[lhs]);
            }
            Nop | Kill => (),
        }
    }

    println!("{}:", ret);
    println!("  mov rsp, rbp");
    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
