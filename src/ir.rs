use std::{collections::HashMap, fmt, sync::Mutex};

use lazy_static::lazy_static;

use crate::parse::{Node, NodeType};
use crate::token::TokenType;

lazy_static! {
    static ref VARS: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
    static ref REGNO: Mutex<usize> = Mutex::new(1);
    static ref BASE_REG: Mutex<usize> = Mutex::new(0);
    static ref BPOFF: Mutex<usize> = Mutex::new(0);
    static ref LABEL: Mutex<usize> = Mutex::new(0);
    static ref IRINFO: [IRInfo; 16] = [
        IRInfo::new(IROp::Add, "+", IRType::RegReg),
        IRInfo::new(IROp::Sub, "-", IRType::RegReg),
        IRInfo::new(IROp::Mul, "*", IRType::RegReg),
        IRInfo::new(IROp::Div, "/", IRType::RegReg),
        IRInfo::new(IROp::Imm, "MOV", IRType::RegImm),
        IRInfo::new(IROp::AddImm, "ADD", IRType::RegImm),
        IRInfo::new(IROp::Mov, "MOV", IRType::RegReg),
        IRInfo::new(IROp::Label, "", IRType::Label),
        IRInfo::new(IROp::Jmp, "", IRType::Label),
        IRInfo::new(IROp::Unless, "UNLESS", IRType::RegLabel),
        IRInfo::new(IROp::Return, "RET", IRType::Reg),
        IRInfo::new(IROp::Alloca, "ALLOCA", IRType::RegImm),
        IRInfo::new(IROp::Load, "LOAD", IRType::RegReg),
        IRInfo::new(IROp::Store, "STORE", IRType::RegReg),
        IRInfo::new(IROp::Kill, "KILL", IRType::Reg),
        IRInfo::new(IROp::Nop, "NOP", IRType::NoArg),
    ];
}

#[derive(Clone)]
pub enum IRType {
    NoArg,
    Reg,
    Label,
    RegReg,
    RegImm,
    RegLabel,
}

#[derive(Clone)]
pub struct IRInfo {
    op: IROp,
    name: &'static str,
    pub ty: IRType,
}

impl IRInfo {
    pub fn new(op: IROp, name: &'static str, ty: IRType) -> Self {
        Self { op, name, ty }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IROp {
    Imm,
    Mov,
    Add,
    AddImm,
    Sub,
    Mul,
    Div,
    Label,
    Jmp,
    Unless,
    Return,
    Alloca,
    Load,
    Store,
    Kill,
    Nop,
}

impl From<NodeType> for IROp {
    fn from(t: NodeType) -> Self {
        match t {
            NodeType::BinOp(op, _, _) => Self::from(op),
            e => panic!("cannot convert: {:?}", e),
        }
    }
}

impl From<TokenType> for IROp {
    fn from(t: TokenType) -> Self {
        match t {
            TokenType::Plus => IROp::Add,
            TokenType::Minus => IROp::Sub,
            TokenType::Mul => IROp::Mul,
            TokenType::Div => IROp::Div,
            e => panic!("cannot convert: {:?}", e),
        }
    }
}

#[derive(Clone)]
pub struct IR {
    pub op: IROp,
    pub lhs: Option<usize>,
    pub rhs: Option<usize>,
}

impl IR {
    fn new(op: IROp, lhs: Option<usize>, rhs: Option<usize>) -> Self {
        Self { op, lhs, rhs }
    }
}

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use IRType::*;

        let info = get_irinfo(self);
        let lhs = self.lhs.unwrap();
        match info.ty {
            Label => writeln!(f, ".L{}=>", lhs),
            Reg => writeln!(f, "{} r{}", info.name, lhs),
            RegReg => writeln!(f, "{} r{}, r{}", info.name, lhs, self.rhs.unwrap()),
            RegImm => writeln!(f, "{} r{}, {}", info.name, lhs, self.rhs.unwrap()),
            RegLabel => writeln!(f, "{} r{}, L{}", info.name, lhs, self.rhs.unwrap()),
            NoArg => writeln!(f, "{}", info.name),
        }
    }
}

pub fn dump_ir(irv: &Vec<IR>) {
    for ir in irv {
        println!("{}", ir);
    }
}

pub fn get_irinfo(ir: &IR) -> IRInfo {
    for info in IRINFO.iter() {
        if info.op == ir.op {
            return info.clone();
        }
    }
    panic!("invalid instruction")
}

fn gen_lval(code: &mut Vec<IR>, node: Node) -> Option<usize> {
    match node.ty {
        NodeType::Ident(name) => {
            if VARS.lock().unwrap().get(&name).is_none() {
                VARS.lock()
                    .unwrap()
                    .insert(name.to_string(), *BPOFF.lock().unwrap());
                *BPOFF.lock().unwrap() += 8;
            }
            let r = Some(*REGNO.lock().unwrap());
            *REGNO.lock().unwrap() += 1;
            let off = *VARS.lock().unwrap().get(&name).unwrap();
            code.push(IR::new(IROp::Mov, r, Some(*BASE_REG.lock().unwrap())));
            code.push(IR::new(IROp::AddImm, r, Some(off)));

            r
        }
        _ => panic!("not a local value"),
    }
}

fn gen_expr(code: &mut Vec<IR>, node: Node) -> Option<usize> {
    match node.ty {
        NodeType::Num(val) => {
            let r = Some(*REGNO.lock().unwrap());
            *REGNO.lock().unwrap() += 1;
            code.push(IR::new(IROp::Imm, r, Some(val as usize)));
            r
        }
        NodeType::Ident(_) => {
            let r = gen_lval(code, node);
            code.push(IR::new(IROp::Load, r, r));
            r
        }
        NodeType::BinOp(op, lhs, rhs) => match op {
            TokenType::Equal => {
                let rhs = gen_expr(code, *rhs);
                let lhs = gen_lval(code, *lhs);
                code.push(IR::new(IROp::Store, lhs, rhs));
                code.push(IR::new(IROp::Kill, rhs, None));
                lhs
            }
            _ => {
                let lhs = gen_expr(code, *lhs);
                let rhs = gen_expr(code, *rhs);
                code.push(IR::new(IROp::from(op), lhs, rhs));
                code.push(IR::new(IROp::Kill, rhs, None));
                lhs
            }
        },
        _ => unreachable!(),
    }
}

fn gen_stmt(code: &mut Vec<IR>, node: Node) {
    match node.ty {
        NodeType::If(cond, then, els_may) => {
            let r = gen_expr(code, *cond);
            let x = Some(*LABEL.lock().unwrap());
            *LABEL.lock().unwrap() += 1;
            code.push(IR::new(IROp::Unless, r, x));
            code.push(IR::new(IROp::Kill, r, None));
            gen_stmt(code, *then);

            if let Some(els) = els_may {
                let y = Some(*LABEL.lock().unwrap());
                *LABEL.lock().unwrap() += 1;
                code.push(IR::new(IROp::Jmp, y, None));
                code.push(IR::new(IROp::Label, x, None));
                gen_stmt(code, *els);
                code.push(IR::new(IROp::Label, y, None));
            } else {
                code.push(IR::new(IROp::Label, x, None));
            }
        }
        NodeType::Return(expr) => {
            let r = gen_expr(code, *expr);
            code.push(IR::new(IROp::Return, r, None));
            code.push(IR::new(IROp::Kill, r, None));
        }
        NodeType::ExprStmt(expr) => {
            let r = gen_expr(code, *expr);
            code.push(IR::new(IROp::Kill, r, None));
        }
        NodeType::CompStmt(stmts) => {
            for n in stmts {
                gen_stmt(code, n);
            }
        }
        e => panic!("unknown code: {:?}", e),
    }
}

pub fn gen_ir(node: Node) -> Vec<IR> {
    let mut code = vec![];

    code.push(IR::new(IROp::Alloca, Some(*BASE_REG.lock().unwrap()), None));
    gen_stmt(&mut code, node);
    code[0].rhs = Some(*BPOFF.lock().unwrap());
    code.push(IR::new(IROp::Kill, Some(*BASE_REG.lock().unwrap()), None));
    code
}
