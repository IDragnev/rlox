use crate::{
    expression,
    scanner::{
        Token,
        TokenType,
    },
    RuntimeValue,
    RuntimeError,
    is_truthy,
    Instance,
    CallableWrapper,
    bind_method,
};
use dumpster::unsync::Gc;
use std::cell::RefCell;
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
        self.look_up_var(&e.name, e.hops)
    }

    fn visit_this(&mut self, e: &expression::This) -> EvalResult {
        self.look_up_var(&e.keyword, e.hops)
    }

    fn visit_assignment(
        &mut self,
        e: &expression::Assignment,
    ) -> EvalResult {
        let v = self.evaluate_expr(&e.value)?;

        let var_exists = self.assign_var(&e.name, &v, e.hops);
        if var_exists {
            Ok(v)
        }
        else {
            Err(RuntimeError::UndefinedVariable(e.name.clone()))
        }
    }

    fn visit_call(&mut self, e: &expression::Call) -> EvalResult {
        let value = self.evaluate_expr(&e.callee)?;

        match value {
            RuntimeValue::Callable(CallableWrapper { callable, closure }) => {
                if callable.arity() != e.args.len() {
                    return Err(RuntimeError::CallableArityMismatch {
                        right_paren: e.right_paren.clone(),
                        expected: callable.arity(),
                        found: e.args.len(),
                    });
                }

                let mut args = Vec::new();
                for a in &e.args {
                    args.push(self.evaluate_expr(a)?);
                }

                callable.call(&args, self, &closure)
            },
            RuntimeValue::Class(class) => {
                let instance = Gc::new(RefCell::new(Instance::new(&class)));

                match class.borrow().methods.get("init") {
                    Some(initializer) => {
                        if initializer.callable.arity() != e.args.len() {
                            return Err(RuntimeError::CallableArityMismatch {
                                right_paren: e.right_paren.clone(),
                                expected: initializer.callable.arity(),
                                found: e.args.len(),
                            });
                        }

                        let mut args = Vec::new();
                        for a in &e.args {
                            args.push(self.evaluate_expr(a)?);
                        }

                        let init = crate::bind_method(initializer, &instance);
                        init.callable.call(&args, self, &init.closure)?;
                    },
                    None => {
                        if e.args.len() != 0 {
                            return Err(RuntimeError::CallableArityMismatch {
                                right_paren: e.right_paren.clone(),
                                expected: 0,
                                found: e.args.len(),
                            });
                        }
                    }
                };

                Ok(RuntimeValue::Instance(instance))
            },
            _ => {
                Err(RuntimeError::NonCallableCalled(e.right_paren.clone()))
            },
        }
    }

    fn visit_get(&mut self, e: &expression::Get) -> EvalResult {
        let expr = self.evaluate_expr(&e.object)?;

        if let RuntimeValue::Instance(instance) = expr {
            instance.borrow()
                .get(&e.name.lexeme, &instance)
                .ok_or(RuntimeError::UndefinedProperty(e.name.clone()))
        }
        else {
            Err(RuntimeError::OnlyInstancesHaveProperties(
                e.name.clone(),
            ))
        }
    }

    fn visit_set(&mut self, e: &expression::Set) -> EvalResult {
        let expr = self.evaluate_expr(&e.object)?;

        if let RuntimeValue::Instance(instance) = expr {
            let v = self.evaluate_expr(&e.value)?;
            instance.borrow_mut().set(&e.name.lexeme, &v);
            Ok(v)
        }
        else {
            Err(RuntimeError::OnlyInstancesHaveProperties(
                e.name.clone(),
            ))
        }
    }

    fn visit_super(&mut self, e: &expression::Super) -> EvalResult {
        let super_class = self.look_up_var(&e.keyword, e.hops_to_super)?;
        if let RuntimeValue::Class(sup) = &super_class {
            let this_token = Token {
                token_type: TokenType::This,
                lexeme: "this".to_owned(),
                literal: None,
                line: 0,
                column: 0
            };
            let instance = self.look_up_var(&this_token, e.hops_to_this)?;
            if let RuntimeValue::Instance(obj) = instance {
                    let method = sup
                    .borrow()
                    .find_method(&e.method.lexeme)
                    .ok_or(RuntimeError::UndefinedProperty(e.method.clone()))?;

                return Ok(RuntimeValue::Callable(bind_method(&method, &obj)));
            }
            else {
                panic!("'this' evaluated wrong");
            }
        }
        else {
            // should never happen
            panic!("'super' evaluated wrong");
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