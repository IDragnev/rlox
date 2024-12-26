mod eval;
mod env;

use crate::{
    expression,
    statement,
    RuntimeValue,
    RuntimeError,
    is_truthy,
};

pub struct Interpreter {
    env: env::EnvStack,
}

pub type ExecResult = Result<(), RuntimeError>;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: env::EnvStack::new(),
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

    fn execute_block(&mut self, s: &Vec<Box<dyn statement::Stmt>>) -> ExecResult {
        self.env.push_env();
        let r = self.execute(s);
        self.env.pop_env();

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

        self.env.define(&s.name.lexeme, &v);

        Ok(())
    }

    fn visit_block(&mut self, s: &statement::Block) -> ExecResult {
        self.execute_block(&s.statements)
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
}
