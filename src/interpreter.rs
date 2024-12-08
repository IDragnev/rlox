use crate::{
    eval::{
        ExprEvalVisitor,
        RuntimeValue,
        RuntimeError,
     },
    expression,
    statement,
};

pub struct Interpreter {
    // no need for box when expr::Visitor is done properly -> directly ExprEvalVisitor
    expr_evaluator: Box<dyn expression::Visitor<Result<RuntimeValue, RuntimeError>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            expr_evaluator: Box::new(ExprEvalVisitor {})
        }
    }

    pub fn interpret_statements(&mut self, statements: &Vec<Box<dyn statement::Stmt>>) {
        for s in statements.iter() {
            s.accept_exec(self);
        }
    }

    fn evaluate_expr(&self, expr: &Box<dyn expression::Expr>) -> Option<RuntimeValue> {
        match expr.accept_rt_value(&self.expr_evaluator) {
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
}

fn stringify(val: &crate::eval::RuntimeValue) -> String {
    use crate::eval::RuntimeValue;

    match val {
        RuntimeValue::Nil => "nil".to_owned(),
        RuntimeValue::Number(n) => n.to_string(),
        RuntimeValue::Bool(b) => b.to_string(),
        RuntimeValue::String(s) => format!("\"{}\"", s),
    }
}
