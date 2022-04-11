use crate::parse::{Node, NodeType};

#[derive(Debug, Clone)]
pub enum IRType {
    IMM,
    MOV,
    RETURN,
    KILL,
    NOP,
    ADD,
    SUB,
    MUL,
    DIV,
}

impl From<NodeType> for IRType {
    fn from(t: NodeType) -> Self {
        match t {
            NodeType::Plus => IRType::ADD,
            NodeType::Minus => IRType::SUB,
            NodeType::Mul => IRType::MUL,
            NodeType::Div => IRType::DIV,
            e => panic!("cannot convert: {:?}", e),
        }
    }
}

#[derive(Clone)]
pub struct IR {
    pub op: IRType,
    pub lhs: usize,
    pub rhs: usize,
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

    assert!(
        node.ty == NodeType::Plus
            || node.ty == NodeType::Minus
            || node.ty == NodeType::Mul
            || node.ty == NodeType::Div
    );

    let (lhs, ins) = gen_ir_sub(v, *node.lhs.unwrap());
    let (rhs, mut ins) = gen_ir_sub(ins, *node.rhs.unwrap());

    ins.push(IR::new(IRType::from(node.ty), lhs, rhs));
    ins.push(IR::new(IRType::KILL, rhs, 0));
    (lhs, ins)
}

pub fn gen_ir(node: Node) -> Vec<IR> {
    let (r, mut ins) = gen_ir_sub(vec![], node);
    ins.push(IR::new(IRType::RETURN, r, 0));
    ins
}