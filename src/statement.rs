use crate::{expression, scanner::Token, RuntimeError};

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

pub struct Block {
    pub statements: Vec<Box<dyn Stmt>>,
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, s: &Expression) -> T;
    fn visit_print(&mut self, s: &Print) -> T;
    fn visit_variable(&mut self, s: &Variable) -> T;
    fn visit_block(&mut self, s: &Block) -> T;
}

type ExecResult = Result<(), RuntimeError>;

pub trait Stmt {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult;
}

impl Stmt for Print {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_print(self)
    }
}

impl Stmt for Expression {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_expr(self)
    }
}

impl Stmt for Variable {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_variable(self)
    }
}

impl Stmt for Block {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_block(self)
    }
}