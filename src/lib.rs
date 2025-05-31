pub mod scanner;
pub mod expression;
pub mod parser;
pub mod statement;
pub mod interpreter;
pub mod resolver;

use scanner::Token;
use statement::StmtEffect;
use std::fmt::Display;
use dumpster::unsync::Gc;
use interpreter::env::Environment;
use std::cell::RefCell;
use std::collections::HashMap;

pub trait Callable: dyn_clone::DynClone + Display {
    fn arity(&self) -> usize;
    fn call(
        &self,
        args: &Vec<RuntimeValue>,
        interp: &mut interpreter::Interpreter,
        closure: &Option<Gc<RefCell<Environment>>>,
    ) -> Result<RuntimeValue, RuntimeError>;
}

#[derive(Clone)]
pub struct CallableWrapper {
    callable: Box<dyn Callable>,
    // not behind a `Callable` implementation because
    // then `Callable` cannot be made into an object
    closure: Option<Gc<RefCell<Environment>>>,
}

dyn_clone::clone_trait_object!(Callable);

#[derive(Clone)]
pub enum RuntimeValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable(CallableWrapper),
    Class(Gc<RefCell<Class>>),
    Instance(Gc<RefCell<Instance>>),
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
    OnlyInstancesHaveProperties(Token),
    UndefinedProperty(Token),
    SuperClassMustBeAClass(Token),
}

pub type RuntimeResult = Result<RuntimeValue, RuntimeError>;

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
    is_initializer: bool,
}

#[derive(Clone)]
pub struct Class {
    pub name: String,
    super_class: Option<Gc<RefCell<Class>>>,
    methods: HashMap<String, CallableWrapper>,
}

impl Class {
    pub fn new(
        name: &str,
        super_class: Option<Gc<RefCell<Class>>>,
        methods: HashMap<String, CallableWrapper>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            super_class,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<CallableWrapper> {
        let mut method = self.methods.get(name).map(|m| m.clone());
        if method.is_none() {
            if let Some(sup) = &self.super_class {
                method = sup.borrow().find_method(name);
            }
        }

        method
    }
}

#[derive(Clone)]
pub struct Instance {
    class: Gc<RefCell<Class>>,
    fields: HashMap<String, RuntimeValue>, 
}

impl Instance {
    pub fn new(class: &Gc<RefCell<Class>>) -> Self {
        Self {
            class: class.clone(),
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str, self_ptr: &Gc<RefCell<Instance>>) -> Option<RuntimeValue> {
        let v = self.fields.get(name).map(|f| f.clone());
        if v.is_some() {
            return v;
        }

        self.class
            .borrow()
            .find_method(name)
            .map(|m| {
                RuntimeValue::Callable(bind_method(&m, self_ptr))
            })
    }

    pub fn set(&mut self, name: &str, v: &RuntimeValue) {
        self.fields.insert(name.to_owned(), v.clone());
    }
}

pub fn bind_method(
    callable_wrapper: &CallableWrapper,
    instance: &Gc<RefCell<Instance>>,
) -> CallableWrapper {
    let mut env = match &callable_wrapper.closure {
        Some(closure) => Environment::child(closure.clone()),
        None => {
            panic!("Trying to bind a global function.");
        }
    };

    env.define("this", &RuntimeValue::Instance(instance.clone()));
    let env = Gc::new(RefCell::new(env));

    CallableWrapper {
        closure: Some(env),
        callable: callable_wrapper.callable.clone(),
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", &self.name)
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<instance of class {}>", &self.class.borrow().name)
    }
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
            Some(StmtEffect::Return(v)) => {
                if self.is_initializer {
                    // workaround: initializer must always return 'this'
                    if let Some(cl) = closure {
                        let instance = cl.borrow()
                            .get("this")
                            .expect("initializer closure without 'this'");
                        return Ok(instance);
                    }
                    else {
                        panic!("initializer without closure");
                    }
                }
                else {
                    return Ok(v)
                }
            }
            None => {
                if self.is_initializer {
                    // workaround: initializer must always return 'this'
                    if let Some(cl) = closure {
                        let instance = cl.borrow()
                            .get("this")
                            .expect("initializer closure without 'this'");
                        return Ok(instance);
                    }
                    else {
                        panic!("initializer without closure");
                    }
                }
                else {
                    return Ok(RuntimeValue::Nil)
                }
            }
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
            RuntimeValue::Callable(CallableWrapper { closure: _, callable }) => callable.fmt(f),
            RuntimeValue::Class(c) => c.borrow().fmt(f),
            RuntimeValue::Instance(i) => i.borrow().fmt(f),
        }
    }
}

unsafe impl dumpster::Trace for RuntimeValue {
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        match self {
            RuntimeValue::Callable(c) => { 
                c.accept(visitor)?
            },
            RuntimeValue::Instance(instance) => {
                instance.accept(visitor)?
            },
            _ => {},
        }

        Ok(())
    }
}

unsafe impl dumpster::Trace for CallableWrapper {
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        if let Some(cl) = &self.closure {
            cl.accept(visitor)?
        }
        Ok(())
    }
}

unsafe impl dumpster::Trace for Class {
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        for (_, value) in &self.methods {
            value.accept(visitor)?;
        }

        Ok(())
    }
}

unsafe impl dumpster::Trace for Instance {
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        self.class.accept(visitor)?;

        for (_, value) in &self.fields {
            value.accept(visitor)?;
        }

        Ok(())
    }
}