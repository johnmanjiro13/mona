#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Num,
    Plus,
    Minus,
    Mul,
    Div,
    Return,
    Semicolon,
}

impl From<char> for TokenType {
    fn from(c: char) -> Self {
        match c {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Mul,
            '/' => TokenType::Div,
            ';' => TokenType::Semicolon,
            e => panic!("unknown token type: {}", e),
        }
    }
}

impl From<String> for TokenType {
    fn from(s: String) -> Self {
        match &*s {
            "return" => TokenType::Return,
            name => panic!("unknown identifier: {}", name),
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
            '+' | '-' | '*' | '/' | ';' => {
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

        // Keyword
        if c.is_alphabetic() || c == '_' {
            let mut name = String::new();
            while let Some(c2) = p.chars().next() {
                p = p.split_off(1);
                if c2.is_alphabetic() || c2.is_ascii_digit() || c2 == '_' {
                    name.push(c2);
                    continue;
                }
                break;
            }
            let token = Token {
                ty: TokenType::from(name),
                input: org.clone(),
                ..Default::default()
            };
            tokens.push(token);
            continue;
        }

        if c.is_ascii_digit() {
            let n = strtol(&mut p);
            let token = Token {
                ty: TokenType::Num,
                input: org.clone(),
                val: n.unwrap() as i32,
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
