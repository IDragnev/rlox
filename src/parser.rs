use crate::scanner::{
    Token,
    TokenType,
};
use crate::expression::{
    Binary,
    Expr,
    Grouping,
    Literal,
    Ternary,
    Unary,
};
use std::iter::Peekable;
use core::slice::Iter;

#[derive(Clone, Debug)]
pub enum ParseErrorType {
    ExpectedToken(TokenType),
    ExpectedExpression,
}

#[derive(Clone, Debug)]
pub struct ParseError {
    error_type: ParseErrorType,
    // to do - add error location info
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

    fn synchronize(iter: &mut Peekable<Iter<'_, Token>>) {
        let error_token = iter.next();
        if let Some(token) = error_token {
            if token.token_type == TokenType::Semicolon {
                // statement end
                return;
            }
        }

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::If |
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::While |
                TokenType::Print |
                TokenType::Class |
                TokenType::Return => {
                    // next statement reached
                    return;
                },
                TokenType::Semicolon => {
                    // statement end
                    let _ = iter.next(); 
                    return;
                },
                _ => {
                    let _ = iter.next(); 
                },
            }
        }
    }

    fn parse_expr(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        self.parse_comma_separated(iter)
    }

    fn parse_comma_separated(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_ternary(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Comma => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_ternary(iter)?;
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

    fn parse_ternary(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_equality(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::QuestionMark => {
                    let _question_mark = iter.next();

                    let left = self.parse_equality(iter)?;
                    if let Some(&token) = iter.peek() {
                        if token.token_type == TokenType::Colon {
                            let _colon = iter.next();
                            let right = self.parse_ternary(iter)?;

                            let ternary = Box::new(Ternary {
                                cond: result,
                                right,
                                left,
                            });
                            result = ternary;
                        }
                        else {
                            return Err(ParseError {
                                error_type: ParseErrorType::ExpectedToken(TokenType::Colon),
                            });
                        }
                    }
                    else {
                        return Err(ParseError {
                            error_type: ParseErrorType::ExpectedToken(TokenType::Colon),
                        });
                    }
                },
                _ => {
                    break;
                }
            }
        }

        Ok(result)
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
                        panic!("Expected string literal");
                    }
                },
                TokenType::Number => {
                    let literal = token.clone().literal.unwrap();
                    if let ScanLiteral::Number(n) = literal {
                        return Ok(Box::new(Literal::Number(n)));
                    }
                    else {
                        panic!("Expected number literal");
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
                            return Err(ParseError {
                                error_type: ParseErrorType::ExpectedToken(TokenType::RightParen),
                            });
                        }
                    }
                    else {
                        return Err(ParseError {
                            error_type: ParseErrorType::ExpectedToken(TokenType::RightParen),
                        });
                    }
                },
                _ => {
                    return Err(ParseError {
                        error_type: ParseErrorType::ExpectedExpression
                    });
                }
            }
        }

        return Err(ParseError {
            error_type: ParseErrorType::ExpectedExpression
        });
    }
}

#[cfg(test)] 

mod tests {
    use crate::{
        expression::{
            self,
            Visitor,
        },
        scanner::scan,
    };
    use super::*;

    // An expression visitor that generates
    // a LISP-like string for a given expression
    struct PrintVisitor {}

    impl Visitor<String> for PrintVisitor {
        fn visit_literal(&self, _: &Box<dyn Visitor<String>>, e: &expression::Literal) -> String {
            use expression::Literal;

            match e {
                Literal::Number(n) => n.to_string(),
                Literal::String(s) => s.clone(),
                Literal::True => "true".to_owned(),
                Literal::False => "false".to_owned(),
                Literal::Nil => "nil".to_owned(),
            }
        }

        fn visit_unary(&self, v: &Box<dyn Visitor<String>>, e: &expression::Unary) -> String {
            format!("({} {})",
                    e.operator.lexeme,
                    e.right.accept_string(v),
            )
        }

        fn visit_binary(&self, v: &Box<dyn Visitor<String>>, e: &expression::Binary) -> String {
            format!("({} {} {})",
                    e.operator.lexeme,
                    e.left.accept_string(v),
                    e.right.accept_string(v),
            )
        }

        fn visit_ternary(&self, v: &Box<dyn Visitor<String>>, e: &expression::Ternary) -> String {
            format!("({} {} {})",
                    e.cond.accept_string(v),
                    e.left.accept_string(v),
                    e.right.accept_string(v),
            )
        }

        fn visit_grouping(&self, v: &Box<dyn Visitor<String>>, e: &expression::Grouping) -> String {
            format!("(group {})", e.0.accept_string(v))
        }
    }

    #[test]
    fn parse_invalid_expression_fails() {
        let parser = Parser::new(&scan("< 10").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("=== 10").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("true ? true ? false : true : false").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("(1 + 2").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("1,").unwrap());
        assert!(parser.parse().is_err());
    }

    #[test]
    fn parse_comma_separated_primary_expressions() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("nil,12.5,\"str\",true,false").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(, (, (, (, nil 12.5) str) true) false)");
        }
    }

    #[test]
    fn parse_grouping() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("(nil)").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(group nil)");
        }
    }

    #[test]
    fn parse_unary() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("---12.5").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(- (- (- 12.5)))");
        }
    }

    #[test]
    fn parse_factor() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("2 * 3 / -2").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(/ (* 2 3) (- 2))");
        }
    }

    #[test]
    fn parse_term() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("2 - 3 + 5 * -2").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(+ (- 2 3) (* 5 (- 2)))");
        }
    }

    #[test]
    fn parse_comparison() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("2 > 3 * 2 - 10").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(> 2 (- (* 3 2) 10))");
        }
    }

    #[test]
    fn parse_equality() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("2 > 3 * 2 - 10 == false").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(== (> 2 (- (* 3 2) 10)) false)");
        }
    }

    #[test]
    fn parse_ternary() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("5 > 2 ? 1 + 3 : 2 * 4").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "((> 5 2) (+ 1 3) (* 2 4))");
        }
    }

    #[test]
    fn parse_nested_ternary() {
        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
        let parser = Parser::new(&scan("true ? true : false ? true : true ? true : false").unwrap());
        let expr = parser.parse();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&visitor);
            assert_eq!(str, "(true true (false true (true true false)))");
        }
    }
}