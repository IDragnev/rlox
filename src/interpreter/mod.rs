mod eval;
pub mod env;

use env::Environment;

use crate::{
    expression,
    statement,
    RuntimeValue,
    RuntimeError,
    is_truthy,
    statement::StmtEffect,
    scanner::Token,
    Class,
    CallableWrapper,
};
use dumpster::{
    Trace,
    unsync::Gc,
    Visitor,
};
use std::{
    cell::RefCell,
    collections::HashMap,
};

pub struct Interpreter {
    globals_env: Gc<RefCell<Environment>>,
    current_env: Gc<RefCell<Environment>>,
}

unsafe impl Trace for Interpreter {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        self.globals_env.accept(visitor)?;
        self.current_env.accept(visitor)?;

        Ok(())
    }
}

pub type ExecResult = Result<Option<StmtEffect>, RuntimeError>;

impl Interpreter {
    pub fn new() -> Self {
        let globals = Gc::new(RefCell::new(
            Environment::root()
        ));

        Interpreter {
            globals_env: globals.clone(),
            current_env: globals,
        }
    }

    pub fn evaluate_expr(&mut self, expr: &Box<dyn expression::Expr>) -> Result<RuntimeValue, RuntimeError> {
        expr.accept_rt_value(self)
    }

    pub fn execute(&mut self, statements: &Vec<Box<dyn statement::Stmt>>) -> ExecResult {
        for s in statements.iter() {
            let effect = self.execute_statement(s)?;
            match effect {
                Some(StmtEffect::Break) |
                Some(StmtEffect::Return(_)) => {
                    return Ok(effect);
                },
                None => { },
            }
        }

        Ok(None)
    }

    pub fn execute_block(
        &mut self,
        s: &Vec<Box<dyn statement::Stmt>>,
        env: Gc<RefCell<Environment>>,
    ) -> ExecResult {
        let prev_env = self.current_env.clone();

        self.current_env = env;
        let r = self.execute(s);

        self.current_env = prev_env;

        r
    }

    fn execute_statement(&mut self, s: &Box<dyn statement::Stmt>) -> ExecResult {
        s.accept_exec(self)
    }

    fn look_up_var(&self, name: &Token, hops: Option<usize>) -> Result<RuntimeValue, RuntimeError> {
        let value = match hops {
            Some(h) => {
                self.current_env
                    .borrow()
                    .get_at(&name.lexeme, h)
            },
            None => {
                self.globals_env
                    .borrow()
                    .get(&name.lexeme)
            }
        };

        value.ok_or(RuntimeError::UndefinedVariable(name.clone()))
             .map(|v| v.clone())
    }

    fn assign_var(&mut self, name: &Token, value: &RuntimeValue, hops: Option<usize>) -> bool {
        match hops {
            Some(h) => {
                self.current_env
                    .borrow_mut()
                    .assign_at(&name.lexeme, value, h)
            },
            None => {
                self.globals_env
                    .borrow_mut()
                    .assign(&name.lexeme, value)
            }
        }
    }
}

impl statement::Visitor<ExecResult> for Interpreter {
    fn visit_expr(&mut self, s: &statement::Expression) -> ExecResult {
        self.evaluate_expr(&s.expr)
            .map(|_| None)
    }

    fn visit_print(&mut self, s: &statement::Print) -> ExecResult {
        let v = self.evaluate_expr(&s.expr)?;
        println!("{}", &v);

        Ok(None)
    }

    fn visit_variable(&mut self, s: &statement::Variable) -> ExecResult {
        let v = match s.initializer {
            None => RuntimeValue::Nil,
            Some(ref init) => {
                self.evaluate_expr(init)?
            },
        };

        self.current_env.borrow_mut().define(&s.name.lexeme, &v);

        Ok(None)
    }

    fn visit_block(&mut self, s: &statement::Block) -> ExecResult {
        let block_env = Gc::new(RefCell::new(
            Environment::child(self.current_env.clone())
        ));
        self.execute_block(&s.statements, block_env)
    }

    fn visit_if(&mut self, s: &statement::If) -> ExecResult {
        let cond = self.evaluate_expr(&s.cond)?;
        if is_truthy(&cond) {
            self.execute_statement(&s.then_branch)
        }
        else {
            match &s.else_branch {
                Some(stmt) => self.execute_statement(stmt),
                None => Ok(None),
            }
        }
    }

    fn visit_while(&mut self, s: &statement::While) -> ExecResult {
        loop {
            let cond = self.evaluate_expr(&s.cond)?;

            if is_truthy(&cond) {
                let effect = self.execute_statement(&s.body)?;
                match effect {
                    Some(StmtEffect::Break) => {
                        break;
                    },
                    Some(StmtEffect::Return(_)) => {
                        return Ok(effect);
                    },
                    None => { },
                }
            }
            else {
                break;
            }
        }

        Ok(None)
    }

    fn visit_function(&mut self, s: &statement::Function) -> ExecResult {
        use crate::{Callable, Function};

        let closure = self.current_env.clone();
        let callable: Box<dyn Callable> = Box::new(Function {
            decl: s.clone(),
            is_initializer: false,
        });
        let value = RuntimeValue::Callable(CallableWrapper {
            callable: callable,
            closure: Some(closure)
        });

        self.current_env.borrow_mut().define(&s.name.lexeme, &value);

        Ok(None)
    }

    fn visit_return(&mut self, s: &statement::Return) -> ExecResult {
        let value = match &s.value {
            Some(expr) => self.evaluate_expr(expr)?,
            None => RuntimeValue::Nil,
        };

        Ok(Some(
            StmtEffect::Return(value)
        ))
    }

    fn visit_break(&mut self, _: &statement::Break) -> ExecResult {
        Ok(Some(StmtEffect::Break))
    }

    fn visit_class(&mut self, s: &statement::Class) -> ExecResult {
        use crate::{ Callable, Function };

        let mut super_class = None;
        if let Some(sup) = &s.super_class {
            let sup_expr: Box<dyn expression::Expr> = Box::new(sup.clone());
            if let RuntimeValue::Class(c) = self.evaluate_expr(&sup_expr)? {
                super_class = Some(c);
            }
            else {
                return Err(RuntimeError::SuperClassMustBeAClass(sup.name.clone()))
            }
        }

        self.current_env.borrow_mut().define(&s.name.lexeme, &RuntimeValue::Nil);

        if let Some(sup) = &super_class {
            let mut env = Environment::child(self.current_env.clone());
            env.define("super", &RuntimeValue::Class(sup.clone()));

            self.current_env = Gc::new(RefCell::new(env));
        }

        let mut class_methods: HashMap<String, CallableWrapper> = HashMap::new();
        for f in &s.methods {
            let is_initializer = f.name.lexeme == "init";
            let closure = self.current_env.clone();
            let callable: Box<dyn Callable> = Box::new(Function {
                decl: f.clone(),
                is_initializer,
            });
            let method = CallableWrapper {
                callable: callable,
                closure: Some(closure)
            };

            class_methods.insert(f.name.lexeme.clone(), method);
        }

        if let Some(_) = &super_class {
            let previous = self.current_env
                .borrow()
                .parent
                .clone()
                .expect("previous environment is non-null");
            self.current_env = previous;
        }

        let class =  RuntimeValue::Class(Gc::new(RefCell::new(
            Class::new(&s.name.lexeme, super_class, class_methods)
        )));
        self.current_env.borrow_mut().assign(&s.name.lexeme, &class);

        Ok(None)
    }
}
