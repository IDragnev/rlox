use crate::{expression::Expr, scanner::Token, RuntimeError};

#[derive(Clone)]
pub struct Expression {
    pub expr: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct Print {
    pub expr: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct Variable {
    pub name: Token,
    pub initializer: Option<Box<dyn Expr>>, 
}

#[derive(Clone)]
pub struct Block {
    pub statements: Vec<Box<dyn Stmt>>,
}

#[derive(Clone)]
pub struct If {
    pub cond: Box<dyn Expr>,
    pub then_branch: Box<dyn Stmt>,
    pub else_branch: Option<Box<dyn Stmt>>,
}

#[derive(Clone)]
pub struct While {
    pub cond: Box<dyn Expr>,
    pub body: Box<dyn Stmt>,
}

#[derive(Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Box<dyn Stmt>>,
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, s: &Expression) -> T;
    fn visit_print(&mut self, s: &Print) -> T;
    fn visit_variable(&mut self, s: &Variable) -> T;
    fn visit_block(&mut self, s: &Block) -> T;
    fn visit_if(&mut self, s: &If) -> T;
    fn visit_while(&mut self, s: &While) -> T;
    fn visit_function(&mut self, s: &Function) -> T;
}

type ExecResult = Result<(), RuntimeError>;

pub trait Stmt: dyn_clone::DynClone {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult;
}

dyn_clone::clone_trait_object!(Stmt);

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

impl Stmt for While {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_while(self)
    }
}

impl Stmt for Function {
    fn accept_exec(&self, v: &mut dyn Visitor<ExecResult>) -> ExecResult {
        v.visit_function(self)
    }
}