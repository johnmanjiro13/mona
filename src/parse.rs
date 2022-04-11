use crate::token::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum NodeType {
    Num(i32),                               // Number literal
    BinOp(TokenType, Box<Node>, Box<Node>), // left-hand, right-hand
    Return(Box<Node>),                      // Return statement
    ExprStmt(Box<Node>),                    // Expression statement
    CompStmt(Vec<Node>),                    // Compound statement
}

#[derive(Debug, Clone)]
pub struct Node {
    pub ty: NodeType,
}

impl Node {
    fn new(op: NodeType) -> Self {
        Self { ty: op }
    }

    fn number(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let t = &tokens[*pos];
        if t.ty != TokenType::Num {
            panic!("number expected, but got {}", t.input);
        }
        *pos += 1;
        Self::new(NodeType::Num(t.val))
    }

    fn mul(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let mut lhs = Self::number(tokens, pos);

        loop {
            if tokens.len() == *pos {
                return lhs;
            }

            let op = tokens[*pos].ty.clone();
            if op != TokenType::Mul && op != TokenType::Div {
                return lhs;
            }
            *pos += 1;
            lhs = Self::new(NodeType::BinOp(
                op,
                Box::new(lhs),
                Box::new(Self::number(tokens, pos)),
            ));
        }
    }

    fn expr(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let mut lhs = Self::mul(tokens, pos);

        loop {
            if tokens.len() == *pos {
                return lhs;
            }

            let op = tokens[*pos].ty.clone();
            if op != TokenType::Plus && op != TokenType::Minus {
                return lhs;
            }
            *pos += 1;
            let rhs = Self::mul(tokens, pos);
            lhs = Self::new(NodeType::BinOp(op, Box::new(lhs), Box::new(rhs)));
        }
    }

    fn stmt(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let mut stmt = vec![];
        loop {
            if tokens.len() == *pos {
                let node = Self::new(NodeType::CompStmt(stmt));
                return node;
            }

            let op = tokens[*pos].ty.clone();
            let e = match op {
                TokenType::Return => {
                    *pos += 1;
                    let expr = Self::expr(tokens, pos);
                    Self::new(NodeType::Return(Box::new(expr)))
                }
                _ => {
                    let expr = Self::expr(tokens, pos);
                    Self::new(NodeType::ExprStmt(Box::new(expr)))
                }
            };
            stmt.push(e);
            matches!(tokens[*pos].ty, TokenType::Semicolon);
            *pos += 1;
        }
    }

    pub fn parse(tokens: &Vec<Token>) -> Self {
        let mut pos = 0;
        Self::stmt(tokens, &mut pos)
    }
}
