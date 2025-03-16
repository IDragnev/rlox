use crate::scanner::{
    Token,
    TokenType,
};
use crate::expression::{
    Binary,
    Expr,
    Grouping,
    Literal,
    Unary,
    Variable,
    Assignment,
    Logical,
    Call,
};
use crate::statement::{self, Stmt};
use std::iter::Peekable;
use core::slice::Iter;

#[derive(Clone, Debug)]
pub enum ParseErrorType {
    ExpectedToken{
        expected: TokenType,
        found: Option<TokenType>,
    },
    ExpectedExpression,
    ExpectedStatement,
    InvalidAssignment,
    ExpectedForLoopInitializerOrSemiColon,
    ExpectedForLoopConditionOrSemiColon,
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub error_type: ParseErrorType,
    pub token: Option<Token>,
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
                error_type: ParseErrorType::ExpectedExpression,
                token: self.tokens.first().map(|t| t.clone()),
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
                TokenType::Fun => self.parse_fun_decl(iter),
                _ => self.parse_statement(iter),
            }
        }
        else {
            self.parse_statement(iter)
        }
    }

    fn parse_fun_decl(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(iter, TokenType::Fun)?;
        self.parse_function(iter)
    }

    fn parse_function(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        // todo: add context to consume_token so error messages
        // are more specific -> 'expected function name' instead of
        // 'expected identifier'.
        let name = self.consume_token(iter, TokenType::Identifier)?;
        let _ = self.consume_token(iter, TokenType::LeftParen)?;
        let params = self.parse_params(iter)?;
        let _ = self.consume_token(iter, TokenType::RightParen)?;
        let body = self.parse_block(iter)?;

        Ok(Box::new(statement::Function {
            name,
            params,
            body,
        }))
    }

    fn parse_params(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Vec<Token>, ParseError> {
        let mut params = Vec::new();

        if let Some(&token) = iter.peek() {
            if token.token_type == TokenType::RightParen {
                return Ok(params);
            }
        }

        loop {
            let p = self.consume_token(iter, TokenType::Identifier)?;
            params.push(p);

            if let None = iter.next_if(|t| t.token_type == TokenType::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_var_decl(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _var = self.consume_token(iter, TokenType::Var)?;
        let name = self.consume_token(iter, TokenType::Identifier)?;

        let mut initializer = None;
        if let Some(_) = iter.next_if(|t| t.token_type == TokenType::Equal) {
            initializer = Some(self.parse_expr(iter)?);
        }
        let _ = self.consume_token(iter, TokenType::Semicolon)?;

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
                TokenType::For => self.parse_for_statement(iter),
                TokenType::While => self.parse_while_statement(iter),
                TokenType::Print => self.parse_print_statement(iter),
                TokenType::Break => self.parse_break_statement(iter),
                TokenType::Return => self.parse_return_statement(iter),
                TokenType::LeftBrace => self.parse_block_statement(iter),
                _ => self.parse_expr_statement(iter),
            }
        }
        else {
            Err(ParseError {
                error_type: ParseErrorType::ExpectedStatement,
                token: None,
            })
        }
    }

    fn parse_print_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(iter, TokenType::Print)?;
        let expr = self.parse_expr(iter)?;
        let _ = self.consume_token(iter, TokenType::Semicolon)?;

        Ok(Box::new(statement::Print{
            expr,
        }))
    }

    fn parse_break_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let brk = self.consume_token(iter, TokenType::Break)?;
        let _ = self.consume_token(iter, TokenType::Semicolon)?;

        Ok(Box::new(statement::Break{
            keyword: brk,
        }))
    }

    fn parse_return_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let ret = self.consume_token(iter, TokenType::Return)?;

        let mut value = None;
        if let Err(_) = self.consume_token(iter, TokenType::Semicolon) {
            let expr = self.parse_expr(iter)?;
            value = Some(expr);

            let _ = self.consume_token(iter, TokenType::Semicolon)?;
        }

        Ok(Box::new(statement::Return{
            keyword: ret,
            value,
        }))
    }

    fn parse_expr_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let expr = self.parse_expr(iter)?;
        let _ = self.consume_token(iter, TokenType::Semicolon)?;

        Ok(Box::new(statement::Expression{
            expr,
        }))
    }

    fn parse_if_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(iter, TokenType::If)?;
        let _ = self.consume_token(iter, TokenType::LeftParen)?;

        let cond = self.parse_expr(iter)?;

        let _ = self.consume_token(iter, TokenType::RightParen)?;

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

    fn parse_for_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(iter, TokenType::For)?;
        let left_paren = self.consume_token(iter, TokenType::LeftParen)?;

        let initializer = match iter.peek() {
            None => {
                return Err(ParseError {
                    error_type: ParseErrorType::ExpectedForLoopInitializerOrSemiColon,
                    token: Some(left_paren.clone()),
                })
            },
            Some(&token) => match token.token_type {
                TokenType::Semicolon => {
                    let _ = iter.next();
                    None
                }
                TokenType::Var => Some(self.parse_var_decl(iter)?),
                _ => Some(self.parse_expr_statement(iter)?),
            },
        };

        let cond = match iter.peek() {
            None => {
                return Err(ParseError {
                    error_type: ParseErrorType::ExpectedForLoopConditionOrSemiColon,
                    token: Some(left_paren.clone()),
                })
            },
            Some(&token) => match token.token_type {
                TokenType::Semicolon => None,
                _ => Some(self.parse_expr(iter)?),
            },
        };
        let _ = self.consume_token(iter, TokenType::Semicolon)?;

        let increment = match iter.peek() {
            Some(&token) => match token.token_type {
                TokenType::RightParen => None,
                _ => Some(self.parse_expr(iter)?),
            },
            None => {
                // this is a parse error but we let the next statement trigger it
                None
            },
        };
        let _ = self.consume_token(iter, TokenType::RightParen)?;

        let mut body = self.parse_statement(iter)?;

        // desugar the for loop into a while loop
        if let Some(inc) = increment {
            body = Box::new(statement::Block {
                statements: vec![
                    body,
                    Box::new(statement::Expression {
                        expr: inc,
                    }),
                ]
            });
        }

        let cond = match cond {
            None => Box::new(Literal::True),
            Some(c) => c,
        };
        body = Box::new(statement::While {
            cond,
            body,
        });

        if let Some(init) = initializer {
            body = Box::new(statement::Block {
                statements: vec![
                    init,
                    body,
                ]
            });
        }

        Ok(body)
    }

    fn parse_while_statement(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Stmt>, ParseError> {
        let _ = self.consume_token(iter, TokenType::While)?;
        let _ = self.consume_token(iter, TokenType::LeftParen)?;

        let cond = self.parse_expr(iter)?;

        let _ = self.consume_token(iter, TokenType::RightParen)?;

        let body = self.parse_statement(iter)?;

        Ok(Box::new(statement::While {
            cond,
            body,
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
        let _ = self.consume_token(iter, TokenType::LeftBrace)?;

        let mut statements = Vec::new();

        while let Some(&token) = iter.peek() {
            if token.token_type == TokenType::RightBrace {
                break;
            }

            let stmt = self.parse_declaration(iter)?;
            statements.push(stmt);
        }

        let _ = self.consume_token(iter, TokenType::RightBrace)?;

        Ok(statements)
    }

    fn parse_expr(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        self.parse_assignment(iter)
    }

    fn parse_assignment(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let left = self.parse_logic_or(iter)?;

        if let Some(eq) = iter.next_if(|t| t.token_type == TokenType::Equal) {
            let right = self.parse_assignment(iter)?;

            // as of now only simple variables can be assigned to,
            // needs to be fixed when classes & member variables are introduced
            if let Some(name) = left.var_name() {
                Ok(Box::new(Assignment {
                    name,
                    value: right,
                    hops: None,
                }))
            }
            else {
                Err(ParseError {
                    error_type: ParseErrorType::InvalidAssignment,
                    token: Some(eq.clone()),
                })
            }
        }
        else {
            Ok(left)
        }
    }

    fn parse_logic_or(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_logic_and(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::Or => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_logic_and(iter)?;
                    let expr = Box::new(Logical {
                        left: result,
                        right,
                        operator,
                    });
                    result = expr;
                },
                _ => {
                    break;
                }
            }
        }
        
        Ok(result)
    }

    fn parse_logic_and(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut result = self.parse_equality(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::And => {
                    let operator = iter.next().unwrap().clone();
                    let right = self.parse_equality(iter)?;
                    let expr = Box::new(Logical {
                        left: result,
                        right,
                        operator,
                    });
                    result = expr;
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

        self.parse_call(iter)
    }

    fn parse_call(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Box<dyn Expr>, ParseError> {
        let mut expr = self.parse_primary(iter)?;

        while let Some(&token) = iter.peek() {
            match token.token_type {
                TokenType::LeftParen => {
                    let _ = self.consume_token(iter, TokenType::LeftParen)?;
                    let args = self.parse_args(iter)?;
                    let right_paren = self.consume_token(iter, TokenType::RightParen)?;

                    expr = Box::new(Call {
                        right_paren,
                        callee: expr,
                        args,
                    })
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_args(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
    ) -> Result<Vec<Box<dyn Expr>>, ParseError> {
        let mut args = Vec::new();

        if let Some(&token) = iter.peek() {
            if token.token_type == TokenType::RightParen {
                return Ok(args);
            }
        }

        loop {
            let expr = self.parse_expr(iter)?;
            args.push(expr);

            if let None = iter.next_if(|t| t.token_type == TokenType::Comma) {
                break;
            }
        }

        Ok(args)
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
                    let _ = self.consume_token(iter, TokenType::RightParen)?;
                    return Ok(Box::new(Grouping(nested)));
                },
                TokenType::Identifier => {
                    return Ok(Box::new(Variable {
                        name: token.clone(),
                        hops: None,
                    }));
                },
                _ => {
                    return Err(ParseError {
                        error_type: ParseErrorType::ExpectedExpression,
                        token: Some(token.clone()),
                    });
                }
            }
        }

        return Err(ParseError {
            error_type: ParseErrorType::ExpectedExpression,
            token: None,
        });
    }

    fn consume_token(
        &self,
        iter: &mut Peekable<Iter<'_, Token>>,
        expected: TokenType,
    ) -> Result<Token, ParseError> {
        if let Some(token) = iter.next_if(|token| token.token_type == expected) {
            Ok(token.clone())
        }
        else {
            let found = iter.peek().map(|&found| found.token_type);

            Err(ParseError {
                token: iter.peek().map(|&t| t.clone()),
                error_type: ParseErrorType::ExpectedToken {
                    expected,
                    found,
                },
            })
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

        fn visit_logical(&mut self, e: &expression::Logical) -> String {
            format!("({} {} {})",
                    e.operator.lexeme,
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

        fn visit_call(&mut self, e: &Call) -> String {
            let args_str = e.args.iter()
                .map(|a| a.accept_string(self))
                .fold(None, |acc, x| {
                    match acc {
                        None => Some(x),
                        Some(y) => Some(y + "," + &x),
                    }
                })
                .unwrap_or_default();

            format!(
                "(call {} {})",
                e.callee.accept_string(self),
                args_str,
            )
        }
    }

    #[test]
    fn parse_invalid_expression_fails() {
        let parser = Parser::new(&scan("< 10").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("=== 10").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("(1 + 2").unwrap());
        assert!(parser.parse().is_err());

        let parser = Parser::new(&scan("1,").unwrap());
        assert!(parser.parse().is_err());
    }

    #[test]
    fn parse_primary_expressions() {
        let primaries = [
            "nil",
            "12.5",
            "\"str\"",
            "true",
            "false",
            "name",
        ];
        for primary in primaries {
            let parser = Parser::new(&scan(primary).unwrap());
            let expr = parser.parse_single_expr();

            assert!(expr.is_ok());
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
    fn parse_logical() {
        let parser = Parser::new(&scan("true or false and true").unwrap());
        let expr = parser.parse_single_expr();

        assert!(expr.is_ok());
        if expr.is_ok() {
            let str = expr.unwrap().accept_string(&mut PrintVisitor{});
            assert_eq!(str, "(or true (and false true))");
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
    fn parse_var_decl_valid_succeeds() {
        let tokens = scan("var x = 10; var y;").unwrap();
        assert!(Parser::new(&tokens).parse().is_ok());
    }

    #[test]
    fn parse_var_decl_invalid_fails() {
        let tokens = scan("var x, y;").unwrap();
        assert!(Parser::new(&tokens).parse().is_err());
    }

    #[test]
    fn parse_expr_stmt() {
        let tokens = scan("2;").unwrap();
        assert!(Parser::new(&tokens).parse().is_ok());
    }

    #[test]
    fn parse_print_stmt() {
        let tokens = scan("print 2;").unwrap();
        assert!(Parser::new(&tokens).parse().is_ok());
    }

    #[test]
    fn parse_block_stmt_valid_succeeds() {
        let tokens = scan("{ { 2; 3; { } } }").unwrap();
        assert!(Parser::new(&tokens).parse().is_ok());
    }

    #[test]
    fn parse_block_stmt_invalid_fails() {
        let invalid_sources = [
            "{ 2 }",
            "{ { 2; 3; }",
        ];

        for src in invalid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_err());
        }
    }

    #[test]
    fn parse_if_stmt_valid_succeeds() {
        let valid_sources = [
            "if (x) print x;",
            "if (x) { print x; }",
            "if (x) print x; else print y;",
            "if (x) { print x; } else { print y; }",
            "if (x) { print x; } else print y;",
            "if (x) print x; else { print y; }",
        ];

        for src in valid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_ok());
        }
    }

    #[test]
    fn parse_if_stmt_invalid_fails() {
        let invalid_sources = [
            "if x print x;",
            "if (x print x;",
            "if x) print x;",
            "if (x) print x; else",
            "if (x;) print x;",
        ];

        for src in invalid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_err());
        }
    }

    #[test]
    fn parse_while_stmt_valid_succeeds() {
        let valid_sources = [
            "while (x) print x;",
            "while (x) { print x; }",
        ];

        for src in valid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_ok());
        }
    }

    #[test]
    fn parse_while_stmt_invalid_fails() {
        let invalid_sources = [
            "while x) { print x; }",
            "while (x { print x; }",
            "while (x;) { print x; }",
        ];

        for src in invalid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_err());
        }
    }

    #[test]
    fn parse_for_stmt_valid_succeeds() {
        let valid_sources = [
            "for (i = 0; i < 5; i = i + 1) print i;",
            "for (var i = 0; i < 5; i = i + 1) print i;",

            "for (; i < 5; i = i + 1) print i;",
            "for (i = 0;; i = i + 1) print i;",
            "for (i = 0; i < 5;) print i;",

            "for (i = 0;;) print i;",
            "for (;true;) print i;",
            "for (;; i = i + 1) print i;",

            "for (;;) print i;",
        ];

        for src in valid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_ok());
        }
    }

    #[test]
    fn parse_for_stmt_invalid_fails() {
        let invalid_sources = [
            "for (i = 0; i >=0 ) {}",
            "for (i = 0; i > 0; i = i + 1 {}",
            "for i = 0; i > 0; i = i + 1) {}",
            "for (i = 0 i > 0; i = i + 1) {}",
            "for () {}",
        ];

        for src in invalid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_err());
        }
    }

    #[test]
    fn parse_call_expr_valid_succeeds() {
        let valid_sources = [
            "myfun()",
            "my_fun()()",
            "my_fun(1)",
            "my_fun(1, 2)",
            "my_fun(1, 2, 3)",
            "my_fun(1, 2, 3)()()",
            "my_fun(1, 2, 3)()(2, 3)",
        ];

        for src in valid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse_single_expr().is_ok());
        }

        let tokens = scan(&"my_fun(1)(2)").unwrap();
        let expr = Parser::new(&tokens).parse_single_expr();

        assert!(expr.is_ok());
        let s = expr.unwrap().accept_string(&mut PrintVisitor{});
        assert_eq!(s, "(call (call my_fun 1) 2)");
    }

    #[test]
    fn parse_call_expr_invalid_fails() {
        let invalid_sources = [
            "myfun(",
            "myfun(1",
            "myfun(1, 2",
            "myfun(1,)",
            "myfun(1,,)",
            "myfun(1)(",
        ];

        for src in invalid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse_single_expr().is_err());
        }
    }

    #[test]
    fn parse_fun_stmt_valid_succeeds() {
        let valid_sources = [
            "fun myfun() { }",
            "fun myfun(a) { print a; }",
            "fun myfun(a, b) { }",
            "fun myfun(a, b, c) { }",
        ];

        for src in valid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_ok());
        }
    }

    #[test]
    fn parse_fun_stmt_invalid_fails() {
        let invalid_sources = [
            "fun myfun() print 1;",
            "fun myfun {}",
            "fun myfun( {}",
            "fun myfun) {}",
            "fun myfun(a,) {}",
            "fun myfun(x = 1) {}",
        ];

        for src in invalid_sources.iter() {
            let tokens = scan(src).unwrap();
            assert!(Parser::new(&tokens).parse().is_err());
        }
    }
}