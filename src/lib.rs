pub mod codegen;
pub mod ir;
pub mod parse;
pub mod regalloc;
pub mod token;

const REGS_N: usize = 8;
const REGS: [&str; REGS_N] = ["rdi", "rsi", "r10", "r11", "r12", "r13", "r14", "r15"];
