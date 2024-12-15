use crate::{
    expression::{
        self,
        Visitor,
    },
    scanner::{
        Token,
        TokenType,
    },
};

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
}

type EvalResult = Result<RuntimeValue, RuntimeError>;

pub fn evaluate(expr: &Box<dyn expression::Expr>) -> EvalResult {
    expr.accept_rt_value(&ExprEvalVisitor{})
}

struct ExprEvalVisitor {}

impl Visitor<EvalResult> for ExprEvalVisitor {
    fn visit_literal(
        &self,
        e: &expression::Literal,
    ) -> EvalResult {
        use expression::Literal as EL;

        let value = match e {
            EL::Number(num) => RuntimeValue::Number(*num),
            EL::String(str) => RuntimeValue::String(str.clone()),
            EL::True => RuntimeValue::Bool(true),
            EL::False => RuntimeValue::Bool(false),
            EL::Nil => RuntimeValue::Nil,
        };

        Ok(value)
    }

    fn visit_unary(
        &self,
        e: &expression::Unary,
    ) -> EvalResult {
        let value = e.right.accept_rt_value(self)?;

        match e.operator.token_type {
            TokenType::Minus => {
                if let RuntimeValue::Number(n) = value {
                    Ok(RuntimeValue::Number(-n))
                }
                else {
                    Err(RuntimeError::UnaryMinusExpectsNumber(e.operator.clone()))
                }
            },
            TokenType::Bang => {
                Ok(RuntimeValue::Bool(
                    is_truthy(&value) == false
                ))
            },
            _ => {
                Err(RuntimeError::UnknownUnaryExpression(e.operator.clone()))
            },
        }
    }

    fn visit_binary(
        &self,
        e: &expression::Binary,
    ) -> EvalResult {
        let left = e.left.accept_rt_value(self)?;
        let right = e.right.accept_rt_value(self)?;

        match e.operator.token_type {
            TokenType::EqualEqual => {
                Ok(RuntimeValue::Bool(
                    are_equal(&left, &right)
                ))
            },
            TokenType::BangEqual => {
                Ok(RuntimeValue::Bool(
                    are_equal(&left, &right) == false
                ))
            },
            TokenType::Less => {
                eval_bin_num_operator(&left, &right, |a, b| RuntimeValue::Bool(a < b), &e.operator)
            },
            TokenType::LessEqual => {
                eval_bin_num_operator(&left, &right, |a, b| RuntimeValue::Bool(a <= b), &e.operator)
            },
            TokenType::Greater => {
                eval_bin_num_operator(&left, &right, |a, b| RuntimeValue::Bool(a > b), &e.operator)
            },
            TokenType::GreaterEqual => {
                eval_bin_num_operator(&left, &right, |a, b| RuntimeValue::Bool(a >= b), &e.operator)
            },
            TokenType::Star => {
                eval_bin_num_operator(&left, &right, |a, b| RuntimeValue::Number(a * b), &e.operator)
            },
            TokenType::Minus => {
                eval_bin_num_operator(&left, &right, |a, b| RuntimeValue::Number(a - b), &e.operator)
            },
            TokenType::Slash => {
                match (&left, &right) {
                    (RuntimeValue::Number(a), RuntimeValue::Number(b)) => {
                        if *b == 0_f64 {
                            Err(RuntimeError::DivisionByZero(e.operator.clone()))
                        }
                        else {
                            Ok(RuntimeValue::Number(a / b))
                        }
                    },
                    _ => {
                        Err(RuntimeError::BinaryOperatorExpectsNumbers(e.operator.clone()))
                    },
                }
            },
            TokenType::Plus => {
                match (&left, &right) {
                    (RuntimeValue::Number(a), RuntimeValue::Number(b)) => {
                        Ok(RuntimeValue::Number(a + b))
                    },
                    (RuntimeValue::String(a), RuntimeValue::String(b)) => {
                        let mut c = a.clone();
                        c += b;
                        Ok(RuntimeValue::String(c))
                    },
                    _ => {
                        Err(RuntimeError::BinaryPlusExpectsTwoNumbersOrTwoStrings(e.operator.clone()))
                    },
                }
            },
            TokenType::Comma => {
                Ok(right)
            },
            _ => {
                Err(RuntimeError::UnknownBinaryExpression(e.operator.clone()))
            }
        }
    }

    fn visit_ternary(
        &self,
        e: &expression::Ternary,
    ) -> EvalResult {
        let cond = e.cond.accept_rt_value(self)?;
        if is_truthy(&cond) {
            e.left.accept_rt_value(self)
        }
        else {
            e.right.accept_rt_value(self)
        }
    }

    fn visit_grouping(
        &self,
        e: &expression::Grouping,
    ) -> EvalResult {
        e.0.accept_rt_value(self)
    }

    fn visit_variable(
        &self,
        e: &expression::Variable,
    ) -> EvalResult {
        unimplemented!() // todo
    }
}

fn is_truthy(value: &RuntimeValue) -> bool {
    match value {
        RuntimeValue::Nil => false,
        RuntimeValue::Bool(b) => *b,
        _ => true,
    }
}

fn eval_bin_num_operator(
    left: &RuntimeValue,
    right: &RuntimeValue,
    f: impl Fn(f64, f64) -> RuntimeValue,
    op: &Token,
) -> EvalResult {
    match (left, right) {
        (RuntimeValue::Number(a), RuntimeValue::Number(b)) => {
            Ok(f(*a, *b))
        },
        _ => {
            Err(RuntimeError::BinaryOperatorExpectsNumbers(op.clone()))
        },
    }
}

fn are_equal(a: &RuntimeValue, b: &RuntimeValue) -> bool {
    match (a, b) {
        (RuntimeValue::Nil, RuntimeValue::Nil) => true,
        (RuntimeValue::Bool(x), RuntimeValue::Bool(y)) => x == y,
        (RuntimeValue::Number(x), RuntimeValue::Number(y)) => x == y,
        (RuntimeValue::String(x), RuntimeValue::String(y)) => x == y,
        _ => false,
    }
}