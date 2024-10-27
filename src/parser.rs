use crate::scanner::{
    Token,
    TokenType,
};
use crate::expression::{
    Expr,
    Binary,
    Grouping,
    Literal,
    Unary
};
use std::iter::Peekable;
use core::slice::Iter;

#[derive(Clone, Debug)]
pub struct ParseError {
}

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        Self {
            tokens: Vec::from(tokens)
        }
    }

    pub fn parse(&self) -> Result<Box<dyn Expr>, ParseError> {
        let mut iter = self.tokens.iter().peekable();
        self.parse_expr(&mut iter)
    }

    fn parse_expr(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        self.parse_equality(iter)
    }

    fn parse_equality(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_comparison(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::EqualEqual | TokenType::BangEqual => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_comparison(iter)?;
                    let binary = Box::new(Binary {
                        left: result,
                        right,
                        operator,
                    });
                    result = binary;
                },
                _ => {
                    break;
                }
            }
        }
        
        Ok(result)
    }

    fn parse_comparison(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_term(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Less |
                TokenType::Greater |
                TokenType::LessEqual |
                TokenType::GreaterEqual => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_term(iter)?;
                    let binary = Box::new(Binary {
                        left: result,
                        right,
                        operator,
                    });
                    result = binary;
                },
                _ => {
                    break;
                }
            }
        }
        
        Ok(result)
    }
    
    fn parse_term(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_factor(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_factor(iter)?;
                    let binary = Box::new(Binary {
                        left: result,
                        right,
                        operator,
                    });
                    result = binary;
                },
                _ => {
                    break;
                }
            }
        }
        
        Ok(result)
    }

    fn parse_factor(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_unary(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Star | TokenType::Slash => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_unary(iter)?;
                    let binary = Box::new(Binary {
                        left: result,
                        right,
                        operator,
                    });
                    result = binary;
                },
                _ => {
                    break;
                }
            }
        }
        
        Ok(result)
    }

    fn parse_unary(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        if let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Bang | TokenType::Minus => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_unary(iter)?;
                    let unary = Box::new(Unary {
                        operator,
                        right,
                    });

                    return Ok(unary);
                },
                _ => { }
            }
        }
        
        self.parse_primary(iter)
    }

    fn parse_primary(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        use crate::scanner::Literal as ScanLiteral;

        if let Some(token) = iter.next() {
            match token.token_type {
                TokenType::False => {
                    return Ok(Box::new(Literal::False));
                },
                TokenType::True => {
                    return Ok(Box::new(Literal::True));
                },
                TokenType::Nil => {
                    return Ok(Box::new(Literal::Nil));
                },
                TokenType::String => {
                    let literal = token.clone().literal.unwrap();
                    if let ScanLiteral::String(s) = literal {
                        return Ok(Box::new(Literal::String(s)));
                    }
                    else {
                        // what do we do? Should not happen
                    }
                },
                TokenType::Number => {
                    let literal = token.clone().literal.unwrap();
                    if let ScanLiteral::Number(n) = literal {
                        return Ok(Box::new(Literal::Number(n)));
                    }
                    else {
                        // what do we do? Should not happen
                    }
                },
                TokenType::LeftParen => {
                    let nested = self.parse_expr(iter)?;
                    if let Some(&token) = iter.peek() {
                        if token.token_type == TokenType::RightParen {
                            let _ = iter.next();
                            return Ok(Box::new(Grouping(nested)));
                        }
                        else {
                            // expected ')'    
                        }
                    }
                    else {
                        // expected ')'
                    }
                },
                _ => {
                    // expected expression
                }
            }
        }

        // expected expression
        Err(ParseError{})
    }

}