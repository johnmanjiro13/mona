use crate::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
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
pub struct Node {
    pub ty: NodeType,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i32,
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
