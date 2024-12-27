use crate::{
    expression,
    scanner::{
        Token,
        TokenType,
    },
    RuntimeValue,
    RuntimeError,
    is_truthy,
};
use super::Interpreter;

type EvalResult = Result<RuntimeValue, RuntimeError>;

impl expression::Visitor<EvalResult> for Interpreter {
    fn visit_literal(
        &mut self,
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
        &mut self,
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
        &mut self,
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

    fn visit_logical(&mut self, e: &expression::Logical) -> EvalResult {
        let left = self.evaluate_expr(&e.left)?;
        let left_truthy = is_truthy(&left);

        match e.operator.token_type {
            TokenType::Or => {
                if left_truthy { 
                    return Ok(left);
                }
            },
            TokenType::And => {
                if left_truthy == false {
                    return Ok(left);
                }
            },
            _ => panic!("expected logical operator"),
        }

        self.evaluate_expr(&e.right)
    }

    fn visit_grouping(
        &mut self,
        e: &expression::Grouping,
    ) -> EvalResult {
        e.0.accept_rt_value(self)
    }

    fn visit_variable(
        &mut self,
        e: &expression::Variable,
    ) -> EvalResult {
        self.env.get(&e.name.lexeme)
            .ok_or(RuntimeError::UndefinedVariable(e.name.clone()))
            .map(|v| v.clone())
    }

    fn visit_assignment(
        &mut self,
        e: &expression::Assignment,
    ) -> EvalResult {
        let v = self.evaluate_expr(&e.value)?;
        let var_exists = self.env.assign(&e.name.lexeme, &v);
        if var_exists {
            Ok(v)
        }
        else {
            Err(RuntimeError::UndefinedVariable(e.name.clone()))
        }
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