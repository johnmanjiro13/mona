use crate::token::{Token, TokenType};

fn expect(t: &Token, ty: TokenType, pos: &mut usize) {
    if t.ty != ty {
        panic!("expected {:?} but got {:?} at {}", ty, t.ty, *pos);
    }
    *pos += 1;
}

fn consume(tokens: &Vec<Token>, ty: TokenType, pos: &mut usize) -> bool {
    let t = &tokens[*pos];
    if t.ty != ty {
        return false;
    }
    *pos += 1;
    true
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Num(i32),                                    // Number literal
    Ident(String),                               // Identifier
    BinOp(TokenType, Box<Node>, Box<Node>),      // left-hand, right-hand
    If(Box<Node>, Box<Node>, Option<Box<Node>>), // condition, then, else
    Return(Box<Node>),                           // Return statement
    ExprStmt(Box<Node>),                         // Expression statement
    CompStmt(Vec<Node>),                         // Compound statement
}

#[derive(Debug, Clone)]
pub struct Node {
    pub ty: NodeType,
}

impl Node {
    fn new(op: NodeType) -> Self {
        Self { ty: op }
    }

    fn term(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let t = &tokens[*pos];
        *pos += 1;
        match t.ty {
            TokenType::Num(val) => Self::new(NodeType::Num(val)),
            TokenType::Ident(ref name) => Self::new(NodeType::Ident(name.to_string())),
            TokenType::LeftParen => {
                let node = Self::assign(tokens, pos);
                expect(&tokens[*pos], TokenType::RightParen, pos);
                node
            }
            _ => panic!("number expected, but got {}", t.input),
        }
    }

    fn mul(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let mut lhs = Self::term(tokens, pos);

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
                Box::new(Self::term(tokens, pos)),
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

    fn assign(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let lhs = Self::expr(tokens, pos);
        if consume(tokens, TokenType::Equal, pos) {
            return Self::new(NodeType::BinOp(
                TokenType::Equal,
                Box::new(lhs),
                Box::new(Self::expr(tokens, pos)),
            ));
        }
        lhs
    }

    fn stmt(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        match tokens[*pos].ty {
            TokenType::If => {
                let mut els = None;
                *pos += 1;
                expect(&tokens[*pos], TokenType::LeftParen, pos);
                let cond = Self::assign(tokens, pos);
                expect(&tokens[*pos], TokenType::RightParen, pos);
                let then = Self::stmt(tokens, pos);
                if consume(tokens, TokenType::Else, pos) {
                    els = Some(Box::new(Self::stmt(tokens, pos)));
                }
                Self::new(NodeType::If(Box::new(cond), Box::new(then), els))
            }
            TokenType::Return => {
                *pos += 1;
                let expr = Self::assign(tokens, pos);
                expect(&tokens[*pos], TokenType::Semicolon, pos);
                Self::new(NodeType::Return(Box::new(expr)))
            }
            _ => {
                let expr = Self::assign(tokens, pos);
                let node = Self::new(NodeType::ExprStmt(Box::new(expr)));
                expect(&tokens[*pos], TokenType::Semicolon, pos);
                node
            }
        }
    }

    fn compound_stmt(tokens: &Vec<Token>, pos: &mut usize) -> Self {
        let mut stmts = vec![];

        loop {
            if tokens.len() == *pos {
                let node = Self::new(NodeType::CompStmt(stmts));
                return node;
            }
            stmts.push(Self::stmt(tokens, pos));
        }
    }

    pub fn parse(tokens: &Vec<Token>) -> Self {
        let mut pos = 0;
        Self::compound_stmt(tokens, &mut pos)
    }
}
