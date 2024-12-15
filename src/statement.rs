use crate::{expression, scanner::Token};

pub struct Expression {
    pub expr: Box<dyn expression::Expr>,
}

pub struct Print {
    pub expr: Box<dyn expression::Expr>,
}

pub struct Variable {
    pub name: Token,
    pub initializer: Option<Box<dyn expression::Expr>>, 
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, s: &Expression) -> T;
    fn visit_print(&mut self, s: &Print) -> T;
    fn visit_variable(&mut self, s: &Variable) -> T;
}

pub trait Stmt {
    fn accept_exec(&self, v: &mut dyn Visitor<()>);
}

impl Stmt for Print {
    fn accept_exec(&self, v: &mut dyn Visitor<()>) {
        v.visit_print(self);
    }
}

impl Stmt for Expression {
    fn accept_exec(&self, v: &mut dyn Visitor<()>) {
        v.visit_expr(self);
    }
}

impl Stmt for Variable {
    fn accept_exec(&self, v: &mut dyn Visitor<()>) {
        v.visit_variable(self);
    }
}