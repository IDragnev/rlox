use crate::{expression::Expr, scanner::Token, RuntimeError};

pub struct Expression {
    pub expr: Box<dyn Expr>,
}

pub struct Print {
    pub expr: Box<dyn Expr>,
}

pub struct Variable {
    pub name: Token,
    pub initializer: Option<Box<dyn Expr>>, 
}

pub struct Block {
    pub statements: Vec<Box<dyn Stmt>>,
}

pub struct If {
    pub cond: Box<dyn Expr>,
    pub then_branch: Box<dyn Stmt>,
    pub else_branch: Option<Box<dyn Stmt>>,
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, s: &Expression) -> T;
    fn visit_print(&mut self, s: &Print) -> T;
    fn visit_variable(&mut self, s: &Variable) -> T;
    fn visit_block(&mut self, s: &Block) -> T;
    fn visit_if(&mut self, s: &If) -> T;
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

impl Stmt for If {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_if(self)
    }
}