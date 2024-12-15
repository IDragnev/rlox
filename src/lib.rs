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