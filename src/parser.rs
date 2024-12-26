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
    Variable,
    Assignment,
};
use crate::statement::{self, Stmt};
use std::iter::Peekable;
use core::slice::Iter;

#[derive(Clone, Debug)]
pub enum ParseErrorType {
    ExpectedToken(TokenType),
    ExpectedExpression,
    ExpectedStatement,
    ExpectedIdentifier,
    InvalidAssignment,
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

    pub fn parse(&self) -> Result<Vec<Box<dyn Stmt>>, Vec<ParseError>> {
        let mut iter = self.tokens.iter().peekable();

        let mut errors = vec![];
        let mut statements = vec![];

        loop {
            if let None = iter.peek() {
                break;
            }

            match self.parse_declaration(&mut iter) {
                Ok(stmt) => {
                    if errors.len() == 0 {
                        statements.push(stmt);
                    }
                },
                Err(e) => {
                    errors.push(e);
                    synchronize(&mut iter);
                }
            }
        }

        if errors.len() > 0 {
            Err(errors)
        }
        else {
            Ok(statements)
        }
    }

    // Parses exactly one expression. If any input is left, it fails.
    // Useful for tests and REPL mode.
    pub fn parse_single_expr(&self) -> Result<Box<dyn Expr>, ParseError> {
        let mut iter = self.tokens.iter().peekable();
        let expr = self.parse_expr(&mut iter)?;

        if iter.len() == 0 {
            Ok(expr)
        }
        else {
            Err(ParseError {
                error_type: ParseErrorType::ExpectedExpression
            })
        }
    }

    fn parse_declaration(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        if let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Var => self.parse_var_decl(iter),
                _ => self.parse_statement(iter),
            }
        }
        else {
            self.parse_statement(iter)
        }
    }

    fn parse_var_decl(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _var = iter.next();
        let name = self.consume_token(
            iter,
            TokenType::Identifier,
            ParseErrorType::ExpectedIdentifier)?;

        let mut initializer = None;
        if let Some(_) = iter.next_if(|t| t.token_type == TokenType::Equal) {
            initializer = Some(self.parse_expr(iter)?);
        }
        let _ = self.consume_token(
            iter,
            TokenType::Semicolon,
            ParseErrorType::ExpectedToken(TokenType::Semicolon))?;

        Ok(Box::new(statement::Variable{
            name,
            initializer,
        }))
    }

    fn parse_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        if let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::If => self.parse_if_statement(iter),
                TokenType::Print => self.parse_print_statement(iter),
                TokenType::LeftBrace => self.parse_block_statement(iter),
                _ => self.parse_expr_statement(iter),
            }
        }
        else {
            Err(ParseError {
                error_type: ParseErrorType::ExpectedStatement
            })
        }
    }

    fn parse_print_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(
            iter,
            TokenType::Print,
            ParseErrorType::ExpectedToken(TokenType::Print))?;
        let expr = self.parse_expr(iter)?;
        let _ = self.consume_token(
            iter,
            TokenType::Semicolon,
            ParseErrorType::ExpectedToken(TokenType::Semicolon))?;

        Ok(Box::new(statement::Print{
            expr,
        }))
    }

    fn parse_expr_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let expr = self.parse_expr(iter)?;
        let _ = self.consume_token(
            iter,
            TokenType::Semicolon,
            ParseErrorType::ExpectedToken(TokenType::Semicolon))?;

        Ok(Box::new(statement::Expression{
            expr,
        }))
    }

    fn parse_if_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(
            iter,
            TokenType::If,
            ParseErrorType::ExpectedToken(TokenType::If))?;
        let _ = self.consume_token(
            iter,
            TokenType::LeftParen,
            ParseErrorType::ExpectedToken(TokenType::LeftParen))?;

        let cond = self.parse_expr(iter)?;

        let _ = self.consume_token(
            iter,
            TokenType::RightParen,
            ParseErrorType::ExpectedToken(TokenType::RightParen))?;

        let then_branch = self.parse_statement(iter)?;

        let mut else_branch = None;
        if let Some(&_) = iter.next_if(|t| t.token_type == TokenType::Else) {
            let else_stmt = self.parse_statement(iter)?;
            else_branch = Some(else_stmt);
        }

        Ok(Box::new(statement::If {
            cond,
            then_branch,
            else_branch,
        }))
    }

    fn parse_block_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let statements = self.parse_block(iter)?;

        Ok(Box::new(statement::Block {
            statements,
        }))
    }

    fn parse_block(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Vec<Box<dyn Stmt>>, ParseError> {
        let _ = self.consume_token(
            iter,
            TokenType::LeftBrace,
            ParseErrorType::ExpectedToken(TokenType::LeftBrace))?;

        let mut statements = Vec::new();

        while let Some(&token) = iter.peek() {
            if token.token_type == TokenType::RightBrace {
                break;
            }

            let stmt = self.parse_declaration(iter)?;
            statements.push(stmt);
        }

        let _ = self.consume_token(
            iter,
            TokenType::RightBrace,
            ParseErrorType::ExpectedToken(TokenType::RightBrace))?;

        Ok(statements)
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
        let mut result = self.parse_assignment(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Comma => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_assignment(iter)?;
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

    fn parse_assignment(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let left = self.parse_ternary(iter)?;

        if let Some(_eq) = iter.next_if(|t| t.token_type == TokenType::Equal) {
            let right = self.parse_assignment(iter)?;

            // as of now only simple variables can be assigned to,
            // needs to be fixed when classes & member variables are introduced
            if let Some(name) = left.var_name() {
                Ok(Box::new(Assignment {
                    name,
                    value: right,
                }))
            }
            else {
                Err(ParseError {
                    error_type: ParseErrorType::InvalidAssignment
                })
            }
        }
        else {
            Ok(left)
        }
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
                TokenType::Identifier => {
                    return Ok(Box::new(Variable {
                        name: token.clone()
                    }));
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

    fn consume_token(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
        t: TokenType,
        err: ParseErrorType,
    ) -> Result<Token, ParseError> {
        if let Some(token) = iter.next_if(|token| token.token_type == t) {
            Ok(token.clone())
        }
        else {
            Err(ParseError { error_type: err })
        }
    }
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
        fn visit_literal(&mut self, e: &expression::Literal) -> String {
            use expression::Literal;

            match e {
                Literal::Number(n) => n.to_string(),
                Literal::String(s) => s.clone(),
                Literal::True => "true".to_owned(),
                Literal::False => "false".to_owned(),
                Literal::Nil => "nil".to_owned(),
            }
        }

        fn visit_unary(&mut self, e: &expression::Unary) -> String {
            format!("({} {})",
                    e.operator.lexeme,
                    e.right.accept_string(self),
            )
        }

        fn visit_binary(&mut self, e: &expression::Binary) -> String {
            format!("({} {} {})",
                    e.operator.lexeme,
                    e.left.accept_string(self),
                    e.right.accept_string(self),
            )
        }

        fn visit_ternary(&mut self, e: &expression::Ternary) -> String {
            format!("({} {} {})",
                    e.cond.accept_string(self),
                    e.left.accept_string(self),
                    e.right.accept_string(self),
            )
        }

        fn visit_grouping(&mut self, e: &expression::Grouping) -> String {
            format!("(group {})", e.0.accept_string(self))
        }

        fn visit_variable(&mut self, e: &expression::Variable) -> String {
            e.name.lexeme.clone()
        }

        fn visit_assignment(&mut self, e: &Assignment) -> String {
            format!("(:= {} {})", e.name.lexeme, e.value.accept_string(self))
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
        let parser = Parser::new(&scan("nil,12.5,\"str\",true,false,name").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(, (, (, (, (, nil 12.5) str) true) false) name)");
        }
    }

    #[test]
    fn parse_grouping() {
        let parser = Parser::new(&scan("(nil)").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(group nil)");
        }
    }

    #[test]
    fn parse_unary() {
        let parser = Parser::new(&scan("---12.5").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(- (- (- 12.5)))");
        }
    }

    #[test]
    fn parse_factor() {
        let parser = Parser::new(&scan("2 * 3 / -2").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(/ (* 2 3) (- 2))");
        }
    }

    #[test]
    fn parse_term() {
        let parser = Parser::new(&scan("2 - 3 + 5 * -2").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(+ (- 2 3) (* 5 (- 2)))");
        }
    }

    #[test]
    fn parse_comparison() {
        let parser = Parser::new(&scan("2 > 3 * 2 - 10").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(> 2 (- (* 3 2) 10))");
        }
    }

    #[test]
    fn parse_equality() {
        let parser = Parser::new(&scan("2 > 3 * 2 - 10 == false").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(== (> 2 (- (* 3 2) 10)) false)");
        }
    }

    #[test]
    fn parse_ternary() {
        let parser = Parser::new(&scan("5 > 2 ? 1 + 3 : 2 * 4").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "((> 5 2) (+ 1 3) (* 2 4))");
        }
    }

    #[test]
    fn parse_nested_ternary() {
        let parser = Parser::new(&scan("true ? true : false ? true : true ? true : false").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(true true (false true (true true false)))");
        }
    }
}