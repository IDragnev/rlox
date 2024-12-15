use crate::{
    eval::{
        self, RuntimeValue
     },
    expression,
    statement,
};

pub struct Interpreter {
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
        }
    }

    pub fn interpret_statements(&mut self, statements: &Vec<Box<dyn statement::Stmt>>) {
        for s in statements.iter() {
            s.accept_exec(self);
        }
    }

    fn evaluate_expr(&self, expr: &Box<dyn expression::Expr>) -> Option<RuntimeValue> {
        match eval::evaluate(expr) {
            Ok(v) => Some(v),
            Err(rt_err) => {
                println!("runtime error: {:?}", rt_err);
                None
            }
        }
    }
}

impl statement::Visitor<()> for Interpreter {
    fn visit_expr(&mut self, s: &statement::Expression) {
        let _ = self.evaluate_expr(&s.expr);
    }

    fn visit_print(&mut self, s: &statement::Print) {
        if let Some(v) = self.evaluate_expr(&s.expr) {
            println!("{}", stringify(&v));
        }
    }

    fn visit_variable(&mut self, s: &statement::Variable) -> () {
        unimplemented!()
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
