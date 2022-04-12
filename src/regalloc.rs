use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::ir::{get_irinfo, IROp, IRType, IR};
use crate::REGS_N;

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

pub fn alloc_regs(irv: &mut Vec<IR>) {
    use IRType::*;
    let irv_len = irv.len();

    *REG_MAP.lock().unwrap() = vec![None; irv_len];

    for i in 0..irv_len {
        let mut ir = irv[i].clone();
        let info = get_irinfo(&ir);

        match info.ty {
            Reg | RegImm | RegLabel => ir.lhs = Some(alloc(ir.lhs.unwrap())),
            RegReg => {
                ir.lhs = Some(alloc(ir.lhs.unwrap()));
                ir.rhs = Some(alloc(ir.rhs.unwrap()));
            }
            _ => (),
        }

        if ir.op == IROp::Kill {
            kill(ir.lhs.unwrap());
            ir.op = IROp::Nop;
        }
        irv[i] = ir;
    }
}
