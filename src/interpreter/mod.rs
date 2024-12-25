mod eval;
mod env;

use crate::{
    expression,
    statement,
    RuntimeValue,
    RuntimeError,
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

    fn execute_block(&mut self, s: &Vec<Box<dyn statement::Stmt>>) -> ExecResult {
        self.env.push_env();
        let r = self.execute(s);
        self.env.pop_env();

        r
    }

    pub fn execute(&mut self, statements: &Vec<Box<dyn statement::Stmt>>) -> ExecResult {
        for s in statements.iter() {
            self.execute_statement(s)?;
        }

        Ok(())
    }

    fn execute_statement(&mut self, s: &Box<dyn statement::Stmt>) -> ExecResult {
        s.accept_exec(self)
    }

    fn evaluate_expr(&mut self, expr: &Box<dyn expression::Expr>) -> Result<RuntimeValue, RuntimeError> {
        expr.accept_rt_value(self)
    }
}

impl statement::Visitor<ExecResult> for Interpreter {
    fn visit_expr(&mut self, s: &statement::Expression) -> ExecResult {
        self.evaluate_expr(&s.expr)
            .map(|_| ())
    }

    fn visit_print(&mut self, s: &statement::Print) -> ExecResult {
        let v = self.evaluate_expr(&s.expr)?;
        println!("{}", stringify(&v));

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
}

fn stringify(val: &RuntimeValue) -> String {
    match val {
        RuntimeValue::Nil => "nil".to_owned(),
        RuntimeValue::Number(n) => n.to_string(),
        RuntimeValue::Bool(b) => b.to_string(),
        RuntimeValue::String(s) => format!("\"{}\"", s),
    }
}
