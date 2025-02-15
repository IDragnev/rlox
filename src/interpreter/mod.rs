mod eval;
pub mod env;

use env::Environment;

use crate::{
    expression,
    statement,
    RuntimeValue,
    RuntimeError,
    is_truthy,
};
use dumpster::{
    Trace,
    unsync::Gc,
    Visitor,
};
use std::cell::RefCell;

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

pub type ExecResult = Result<(), RuntimeError>;

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
            self.execute_statement(s)?;
        }

        Ok(())
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
}

impl statement::Visitor<ExecResult> for Interpreter {
    fn visit_expr(&mut self, s: &statement::Expression) -> ExecResult {
        self.evaluate_expr(&s.expr)
            .map(|_| ())
    }

    fn visit_print(&mut self, s: &statement::Print) -> ExecResult {
        let v = self.evaluate_expr(&s.expr)?;
        println!("{}", &v);

        Ok(())
    }

    fn visit_variable(&mut self, s: &statement::Variable) -> ExecResult {
        let v = match s.initializer {
            None => RuntimeValue::Nil,
            Some(ref init) => {
                self.evaluate_expr(init)?
            },
        };

        self.current_env.borrow_mut().define(&s.name.lexeme, &v);

        Ok(())
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
                None => Ok(()),
            }
        }
    }

    fn visit_while(&mut self, s: &statement::While) -> ExecResult {
        loop {
            let cond = self.evaluate_expr(&s.cond)?;

            if is_truthy(&cond) {
                self.execute_statement(&s.body)?;
            }
            else {
                break;
            }
        }

        Ok(())
    }

    fn visit_function(&mut self, s: &statement::Function) -> ExecResult {
        use crate::{Callable, Function};

        let closure = self.current_env.clone();
        let callable: Box<dyn Callable> = Box::new(Function {
            decl: s.clone(),
        });
        let value = RuntimeValue::Callable {
            callable: callable,
            closure: Some(closure)
        };

        self.current_env.borrow_mut().define(&s.name.lexeme, &value);

        Ok(())
    }
}
