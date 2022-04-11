#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Num(i32),      // Number literal
    Ident(String), // Identifier
    Plus,          // +
    Minus,         // -
    Mul,           // *
    Div,           // /
    If,            // if
    Return,        // return
    Semicolon,     // ;
    LeftParen,     // ()
    RightParen,    // )
    Equal,         // =
}

impl From<char> for TokenType {
    fn from(c: char) -> Self {
        use TokenType::*;
        match c {
            '+' => Plus,
            '-' => Minus,
            '*' => Mul,
            '/' => Div,
            ';' => Semicolon,
            '=' => Equal,
            '(' => LeftParen,
            ')' => RightParen,
            e => panic!("unknown token type: {}", e),
        }
    }
}

impl From<String> for TokenType {
    fn from(s: String) -> Self {
        match &*s {
            "return" => TokenType::Return,
            "if" => TokenType::If,
            name => TokenType::Ident(name.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenType,
    pub input: String,
}

pub fn scan(mut p: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let org = p.clone();
    while let Some(c) = p.chars().next() {
        if c.is_whitespace() {
            p = p.split_off(1);
            continue;
        }

        // Single-letter tokens
        match c {
            '+' | '-' | '*' | '/' | ';' | '=' | '(' | ')' => {
                let token = Token {
                    ty: TokenType::from(c),
                    input: org.clone(),
                };
                p = p.split_off(1);
                tokens.push(token);
                continue;
            }
            _ => (),
        }

        // Identifier
        if c.is_alphabetic() || c == '_' {
            let mut name = String::new();
            while let Some(c2) = p.chars().next() {
                if c2.is_alphabetic() || c2.is_ascii_digit() || c2 == '_' {
                    p = p.split_off(1);
                    name.push(c2);
                    continue;
                }
                break;
            }
            let token = Token {
                ty: TokenType::from(name),
                input: org.clone(),
            };
            tokens.push(token);
            continue;
        }

        if c.is_ascii_digit() {
            let n = strtol(&mut p);
            let token = Token {
                ty: TokenType::Num(n.unwrap() as i32),
                input: org.clone(),
            };
            tokens.push(token);
            continue;
        }

        panic!("cannot tokenize: {}", p);
    }
    tokens
}

pub fn tokenize(p: String) -> Vec<Token> {
    scan(p)
}

fn strtol(s: &mut String) -> Option<i64> {
    if s.is_empty() {
        return None;
    }

    let mut pos = 0;

    for c in s.chars() {
        if !c.is_ascii_digit() {
            break;
        }
        pos += 1;
    }

    let t = s.drain(..pos).collect::<String>();
    Some(t.parse::<i64>().unwrap())
}
