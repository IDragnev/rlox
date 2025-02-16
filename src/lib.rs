pub mod scanner;
pub mod expression;
pub mod parser;
pub mod statement;
pub mod interpreter;

use scanner::Token;
use statement::StmtEffect;
use std::fmt::Display;
use dumpster::unsync::Gc;
use interpreter::env::Environment;
use std::cell::RefCell;

pub trait Callable: dyn_clone::DynClone + Display {
    fn arity(&self) -> usize;
    fn call(
        &self,
        args: &Vec<RuntimeValue>,
        interp: &mut interpreter::Interpreter,
        closure: &Option<Gc<RefCell<Environment>>>,
    ) -> Result<RuntimeValue, RuntimeError>;
}

dyn_clone::clone_trait_object!(Callable);

#[derive(Clone)]
pub enum RuntimeValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable {
        callable: Box<dyn Callable>,
        // not behind a `Callable` implementation because
        // then `Callable` cannot be made into an object
        closure: Option<Gc<RefCell<Environment>>>,
    },
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
    NonCallableCalled(Token),
    CallableArityMismatch{
        right_paren: Token,
        expected: usize,
        found: usize,
    },
}

pub fn is_truthy(value: &RuntimeValue) -> bool {
    match value {
        RuntimeValue::Nil => false,
        RuntimeValue::Bool(b) => *b,
        _ => true,
    }
}

#[derive(Clone)]
pub struct Function {
    pub decl: statement::Function,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun {}>", &self.decl.name.lexeme)
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.decl.params.len()
    }

    fn call(
        &self,
        args: &Vec<RuntimeValue>,
        interp: &mut interpreter::Interpreter,
        closure: &Option<Gc<RefCell<Environment>>>
        ) -> Result<RuntimeValue, RuntimeError> {
        let fun_env = match closure {
            Some(c) => {
                Gc::new(RefCell::new(
                    Environment::child(c.clone())
                ))
            },
            None => {
                Gc::new(RefCell::new(
                    Environment::root()
                ))
            }
        };

        for (i, a) in args.iter().enumerate() {
            let name = &self.decl.params[i].lexeme;
            fun_env.borrow_mut().define(name, a);
        }

        let effect = interp.execute_block(&self.decl.body, fun_env)?;
        match effect {
            Some(StmtEffect::Break) => panic!("break propagated to fuction"),
            Some(StmtEffect::Return(v)) => return Ok(v),
            None => return Ok(RuntimeValue::Nil),
        }
    }
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Nil => write!(f, "nil"),
            RuntimeValue::Bool(b) => write!(f, "{}", b),
            RuntimeValue::Number(n) => write!(f, "{}", n),
            RuntimeValue::String(s) => write!(f, "\"{}\"", s),
            RuntimeValue::Callable{ closure: _, callable } => callable.fmt(f)
        }
    }
}