use std::process;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Num,
    Plus,
    Minus,
    Mul,
    Div,
}

impl From<char> for TokenType {
    fn from(c: char) -> Self {
        match c {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Mul,
            '/' => TokenType::Div,
            e => panic!("unknown token type: {}", e),
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Num
    }
}

#[derive(Default, Debug)]
pub struct Token {
    pub ty: TokenType,
    pub val: i32,
    pub input: String,
}

pub fn tokenize(mut p: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let org = p.clone();
    while let Some(c) = p.chars().next() {
        if c.is_whitespace() {
            p = p.split_off(1);
            continue;
        }

        match c {
            '+' | '-' | '*' | '/' => {
                let token = Token {
                    ty: TokenType::from(c),
                    input: org.clone(),
                    ..Default::default()
                };
                p = p.split_off(1);
                tokens.push(token);
                continue;
            }
            _ => (),
        }

        if c.is_ascii_digit() {
            let (n, remaining) = strtol(&p);
            p = remaining;
            let token = Token {
                ty: TokenType::Num,
                input: org.clone(),
                val: n.unwrap() as i32,
            };
            tokens.push(token);
            continue;
        }

        eprintln!("cannot tokenize: {}", p);
        process::exit(1);
    }
    tokens
}

pub fn strtol(s: &String) -> (Option<i64>, String) {
    if s.is_empty() {
        return (None, s.clone());
    }

    let mut pos = 0;
    let mut remaining = s.clone();
    let len = s.len();

    while len != pos {
        if !s.chars().nth(pos).unwrap().is_ascii_digit() {
            break;
        }
        pos += 1;
    }

    if len == pos {
        (Some(remaining.parse::<i64>().unwrap()), "".into())
    } else {
        let t: String = remaining.drain(..pos).collect();
        (Some(t.parse::<i64>().unwrap()), remaining)
    }
}
