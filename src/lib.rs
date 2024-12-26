pub mod scanner;
pub mod expression;
pub mod parser;
pub mod statement;
pub mod interpreter;

use scanner::Token;

#[derive(Clone)]
pub enum RuntimeValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub enum RuntimeError {
    UnknownUnaryExpression(Token),
    UnknownBinaryExpression(Token),
    UnaryMinusExpectsNumber(Token),
    BinaryOperatorExpectsNumbers(Token),
    BinaryPlusExpectsTwoNumbersOrTwoStrings(Token),
    DivisionByZero(Token),
    UndefinedVariable(Token),
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Nil => write!(f, "nil"),
            RuntimeValue::Bool(b) => write!(f, "{}", b),
            RuntimeValue::Number(n) => write!(f, "{}", n),
            RuntimeValue::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

pub fn is_truthy(value: &RuntimeValue) -> bool {
    match value {
        RuntimeValue::Nil => false,
        RuntimeValue::Bool(b) => *b,
        _ => true,
    }
}