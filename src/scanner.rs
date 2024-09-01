use std::{
    iter::Peekable,
    str::Chars,
    collections::HashMap,
};

#[derive(Copy, Clone, Debug)]
pub enum TokenType {
    // single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
}

#[derive(Clone, Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
}

impl Token {
    fn single_character(token_type: TokenType, c: char) -> Self {
        Token {
            token_type,
            lexeme: c.to_string(),
            literal: None,
        }
    }

    fn two_character(token_type: TokenType, c1: char, c2: char) -> Self {
        let mut lexeme = c1.to_string();
        lexeme.push(c2);

        Token {
            token_type,
            lexeme,
            literal: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct TokenError {
    line: u64,
    column: u64,
    error: TokenErrorType,
}

#[derive(Clone, Debug)]
pub enum TokenErrorType {
    UnexpectedCharacter,
    UnterminatedString,
}

pub enum ScanError {
    NonAsciiCharacterFound,
    TokenError(Vec<TokenError>)
}

impl std::fmt::Debug for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonAsciiCharacterFound => write!(f, "Non-ascii character found."),
            Self::TokenError(v) => write!(f, "Token errors: {:?}", v),
        }
    }
}

pub fn scan(source: &str) -> Result<Vec<Token>, ScanError> {
    if source.is_ascii() {
        scan_ascii(source).map_err(|v| ScanError::TokenError(v))
    }
    else {
        Err(ScanError::NonAsciiCharacterFound)
    }
}

fn scan_ascii(source: &str) -> Result<Vec<Token>, Vec<TokenError>> {
    let keywords = HashMap::from([
        ("and".to_owned(),    TokenType::And),
        ("class".to_owned(),  TokenType::Class),
        ("else".to_owned(),   TokenType::Else),
        ("false".to_owned(),  TokenType::False),
        ("for".to_owned(),    TokenType::For),
        ("fun".to_owned(),    TokenType::Fun),
        ("if".to_owned(),     TokenType::If),
        ("nil".to_owned(),    TokenType::Nil),
        ("or".to_owned(),     TokenType::Or),
        ("print".to_owned(),  TokenType::Print),
        ("return".to_owned(), TokenType::Return),
        ("super".to_owned(),  TokenType::Super),
        ("this".to_owned(),   TokenType::This),
        ("true".to_owned(),   TokenType::True),
        ("var".to_owned(),    TokenType::Var),
        ("while".to_owned(),  TokenType::While),
    ]);

    let mut token_result = Vec::new();
    let mut error_result = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_result = scan_ascii_line(line_num as u64, line, &keywords);

        if error_result.len() > 0 {
            if let Err(v) = line_result {
                error_result.extend(v.into_iter());
            }
        }
        else {
            match line_result {
                Ok(tokens) => token_result.extend(tokens.into_iter()),
                Err(errors) => error_result.extend(errors.into_iter()),
            }
        }
    }

    if error_result.len() > 0 {
        Err(error_result)
    }
    else {
        Ok(token_result)
    }
}

fn scan_ascii_line(line_num: u64, line: &str, keywords: &HashMap<String, TokenType>) -> Result<Vec<Token>, Vec<TokenError>> {
    // let mut line = line.enumerate();
    let mut chars = line.chars().enumerate().peekable();

    let mut token_result = Vec::new();
    let mut error_result = Vec::new();
    let mut push_token = |t| { token_result.push(t) };
    let mut push_error = |l, c, e| { 
        error_result.push(TokenError {
            line: l + 1,
            column: (c + 1) as u64,
            error: e,
        })
    };

    loop {
        let current = chars.next();
        if current.is_none() { 
            break;
        }

        let (col, c) = current.unwrap();
        match c {
            '(' => {
                push_token(Token::single_character(TokenType::LeftParen,  c))
            },
            ')' => {
                push_token(Token::single_character(TokenType::RightParen, c))
            },
            '{' => {
                push_token(Token::single_character(TokenType::LeftBrace, c))
            },
            '}' => {
                push_token(Token::single_character(TokenType::RightBrace, c))
            },
            ',' => {
                push_token(Token::single_character(TokenType::Comma, c))
            },
            '.' => {
                push_token(Token::single_character(TokenType::Dot, c))
            },
            '-' => {
                push_token(Token::single_character(TokenType::Minus, c))
            },
            '+' => {
                push_token(Token::single_character(TokenType::Plus, c))
            },
            ';' => {
                push_token(Token::single_character(TokenType::Semicolon, c))
            },
            '*' => {
                push_token(Token::single_character(TokenType::Star, c))
            },
            '!' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::BangEqual, c, cc));
                }
                else {
                    push_token(Token::single_character(TokenType::Bang, c));
                }
            },
            '=' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::EqualEqual, c, cc));
                }
                else {
                    push_token(Token::single_character(TokenType::Equal, c));
                }
            },
            '<' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::LessEqual, c, cc));
                }
                else {
                    push_token(Token::single_character(TokenType::Less, c));
                }
            },
            '>' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::GreaterEqual, c, cc));
                }
                else {
                    push_token(Token::single_character(TokenType::Greater, c));
                }
            },
            '/' => {
                if let Some(&(_, '/')) = chars.peek() {
                    // ignore comments
                    break;
                }
                else {
                    push_token(Token::single_character(TokenType::Slash, c));
                }
            },
            '\t'|'\r'|' ' => {
                // ignore whitespace
            },
            '"' => {
                let mut terminated = false;
                let mut lexeme = c.to_string();
                while let Some((_, cc)) = chars.next() {
                    lexeme.push(cc);
                    if cc == '"' {
                        terminated = true;
                        break;
                    }
                }

                if terminated == false {
                    push_error(line_num, col, TokenErrorType::UnterminatedString);
                }
                else {
                    let literal = lexeme[1..lexeme.len() - 1].to_string();
                    push_token(Token{
                        token_type: TokenType::String,
                        lexeme,
                        literal: Some(Literal::String(literal)),
                    })
                }
            },
            _ => {
                if c.is_ascii_digit() {
                    let mut lexeme = c.to_string();

                    // consume all digits
                    while let Some((_,d)) = chars.next_if(|&(_, d)| d.is_ascii_digit()) {
                        lexeme.push(d);
                    }

                    if let Some(&(_, '.')) = chars.peek() {
                        let mut chars2 = chars.clone();
                        let _ = chars2.next();

                        // look past the dot if there are more digits to consume
                        if let Some(&(_, d)) = chars2.peek() {
                            if d.is_ascii_digit() {
                                let (_, dot) = chars.next().unwrap();
                                lexeme.push(dot);

                                while let Some((_,d)) = chars.next_if(|&(_, d)| d.is_ascii_digit()) {
                                    lexeme.push(d);
                                }
                            }
                        }
                    }

                    let value = lexeme.clone().parse::<f64>().unwrap();

                    push_token(Token {
                        token_type: TokenType::Number,
                        lexeme: lexeme,
                        literal: Some(Literal::Number(value)),
                    });

                }
                else if is_ascii_alpha(c) {
                    let mut lexeme = c.to_string();
                    while let Some((_,d)) = chars.next_if(|&(_, d)| is_ascii_alphanumeric(d)) {
                        lexeme.push(d);
                    }

                    let mut token_type = TokenType::Identifier;
                    if let Some((_, t)) = keywords.get_key_value(&lexeme) {
                        token_type = *t;
                    }

                    push_token(Token {
                        token_type,
                        lexeme,
                        literal: None,
                    })
                } 
                else {
                    push_error(line_num, col, TokenErrorType::UnexpectedCharacter);
                }
            },
        };
    }

    if error_result.len() > 0 {
        Err(error_result)
    }
    else {
        Ok(token_result)
    }
    
}

fn is_ascii_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ascii_alphanumeric(c: char) -> bool {
    is_ascii_alpha(c) || c.is_ascii_digit()
}