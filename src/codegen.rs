use crate::ir::{IRType, IR};
use crate::REGS;

pub fn gen_x86(irv: Vec<IR>) {
    use IRType::*;
    for ir in irv {
        match ir.op {
            IMM => println!("  mov {}, {}", REGS[ir.lhs], ir.rhs),
            MOV => println!("  mov {}, {}", REGS[ir.lhs], REGS[ir.rhs]),
            RETURN => {
                println!("  mov rax, {}", REGS[ir.lhs]);
                println!("  ret");
            }
            ADD => println!("  add {}, {}", REGS[ir.lhs], REGS[ir.rhs]),
            SUB => println!("  sub {}, {}", REGS[ir.lhs], REGS[ir.rhs]),
            MUL => {
                println!("  mov rax, {}", REGS[ir.rhs]);
                println!("  mul {}", REGS[ir.lhs]);
                println!("  mov {}, rax", REGS[ir.lhs]);
            }
            DIV => {
                println!("  mov rax, {}", REGS[ir.lhs]);
                println!("  cqo");
                println!("  div {}", REGS[ir.rhs]);
                println!("  mov {}, rax", REGS[ir.lhs]);
            }
            NOP | KILL => (),
        }
    }
}
