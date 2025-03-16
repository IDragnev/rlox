use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    Break,
    Super,
    This,
    True,
    Var,
    While,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u64,
    pub column: u64,
}

impl Token {
    pub fn single_character(
        token_type: TokenType,
        c: char,
        line: u64,
        column: u64,
    ) -> Self {
        Token {
            token_type,
            lexeme: c.to_string(),
            literal: None,
            line,
            column,
        }
    }

    pub fn two_character(
        token_type: TokenType,
        c1: char,
        c2: char,
        line: u64,
        column: u64,
    ) -> Self {
        let mut lexeme = c1.to_string();
        lexeme.push(c2);

        Token {
            token_type,
            lexeme,
            literal: None,
            line,
            column,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenError {
    pub line: u64,
    pub column: u64,
    pub error: TokenErrorType,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenErrorType {
    UnexpectedCharacter,
    UnterminatedString,
}

pub enum ScanError {
    NonAsciiCharacterFound,
    TokenError(Vec<TokenError>)
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
    assert!(source.is_ascii(), "expected ascii source");

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
        ("break".to_owned(),  TokenType::Break),
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

fn scan_ascii_line(
    line_num: u64,
    line: &str,
    keywords: &HashMap<String, TokenType>,
) -> Result<Vec<Token>, Vec<TokenError>> {
    assert!(line.is_ascii(), "expected ascii source");

    let mut chars = line.chars().enumerate().peekable();

    let mut token_result = Vec::new();
    let mut error_result = Vec::new();
    let mut push_token = |t| { token_result.push(t) };
    let mut push_error = |l, c, e| { 
        error_result.push(TokenError {
            line: l + 1,
            column: c + 1,
            error: e,
        })
    };

    loop {
        let current = chars.next();
        if current.is_none() { 
            break;
        }

        let (col, c) = current.unwrap();
        let col = col as u64;
        match c {
            '(' => {
                push_token(Token::single_character(TokenType::LeftParen,  c, line_num, col))
            },
            ')' => {
                push_token(Token::single_character(TokenType::RightParen, c, line_num, col))
            },
            '{' => {
                push_token(Token::single_character(TokenType::LeftBrace, c, line_num, col))
            },
            '}' => {
                push_token(Token::single_character(TokenType::RightBrace, c, line_num, col))
            },
            ',' => {
                push_token(Token::single_character(TokenType::Comma, c, line_num, col))
            },
            '.' => {
                push_token(Token::single_character(TokenType::Dot, c, line_num, col))
            },
            '-' => {
                push_token(Token::single_character(TokenType::Minus, c, line_num, col))
            },
            '+' => {
                push_token(Token::single_character(TokenType::Plus, c, line_num, col))
            },
            ';' => {
                push_token(Token::single_character(TokenType::Semicolon, c, line_num, col))
            },
            '*' => {
                push_token(Token::single_character(TokenType::Star, c, line_num, col))
            },
            '!' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::BangEqual, c, cc, line_num, col));
                }
                else {
                    push_token(Token::single_character(TokenType::Bang, c, line_num, col));
                }
            },
            '=' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::EqualEqual, c, cc, line_num, col));
                }
                else {
                    push_token(Token::single_character(TokenType::Equal, c, line_num, col));
                }
            },
            '<' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::LessEqual, c, cc, line_num, col));
                }
                else {
                    push_token(Token::single_character(TokenType::Less, c, line_num, col));
                }
            },
            '>' => {
                if let Some(&(_, '=')) = chars.peek() {
                    let (_, cc) = chars.next().unwrap();
                    push_token(Token::two_character(TokenType::GreaterEqual, c, cc, line_num, col));
                }
                else {
                    push_token(Token::single_character(TokenType::Greater, c, line_num, col));
                }
            },
            '/' => {
                if let Some(&(_, '/')) = chars.peek() {
                    // ignore comments
                    break;
                }
                else {
                    push_token(Token::single_character(TokenType::Slash, c, line_num, col));
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
                        line: line_num + 1,
                        column: col + 1,
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
                        line: line_num + 1,
                        column: col + 1,
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
                        line: line_num + 1,
                        column: col + 1,
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

#[cfg(test)] 
mod tests {
    use super::*;

    #[test]
    fn scan_non_ascii_fails() {
        let source = "var x = ⊥";
        assert!(matches!(scan(source), Err(ScanError::NonAsciiCharacterFound)));
    }

    #[test]
    fn scan_unterminated_string_fails() {
        let source = "\"hello world";
        assert!(scan(source).is_err());
    }

    #[test]
    fn scan_two_line_string_fails() {
        let source = "\"hello world\n\"";
        assert!(scan(source).is_err());
    }

    #[test]
    fn scan_unexpected_character_fails() {
        let source = "^";
        assert!(scan(source).is_err());
    }

    #[test]
    fn scan_valid_tokens() {
        let source = "
            var x = 10;
            var y = 20;
            var z = nil;
            if (x > y) {
                z = x - y;
            }
            else {
                z = x + y;
            }
            var w = x * y;

            while (w > 0) {
                w = w - 1;
            }

            var logicals = true or false and true;
            var comma = 1, 2, (5 + 3);

            class A {
                fun f() { }
            }
        ";
        assert!(scan(source).is_ok());
    }

    #[test]
    fn scan_invalid_code_reports_correct_position() {
        let source = "var x = 0;\nvar y = 2;\nx = x & y;";
        let result = scan(source);

        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, ScanError::TokenError(_)));

            if let ScanError::TokenError(v) = e {
                assert!(v.len() == 1);

                let TokenError { line, column, error } = v[0];
                assert!(line == 3);
                assert!(column == 7);
                assert!(error == TokenErrorType::UnexpectedCharacter);
            }
        }
    }
}