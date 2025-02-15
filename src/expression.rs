use std::boxed::Box;
use crate::scanner::Token;
use crate::{
    RuntimeValue,
    RuntimeError,
};

#[derive(Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}

#[derive(Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct Binary {
    pub left: Box<dyn Expr>,
    pub right: Box<dyn Expr>,
    pub operator: Token,
}

#[derive(Clone)]
pub struct Logical {
    pub left: Box<dyn Expr>,
    pub right: Box<dyn Expr>,
    pub operator: Token,
}

#[derive(Clone)]
pub struct Grouping(pub Box<dyn Expr>);

#[derive(Clone)]
pub struct Variable {
    pub name: Token,
}

#[derive(Clone)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct Call {
    pub right_paren: Token,
    pub callee: Box<dyn Expr>,
    pub args: Vec<Box<dyn Expr>>,
}

pub trait Visitor<T> {
    fn visit_literal(&mut self, e: &Literal) -> T;
    fn visit_unary(&mut self, e: &Unary) -> T;
    fn visit_binary(&mut self, e: &Binary) -> T;
    fn visit_logical(&mut self, e: &Logical) -> T;
    fn visit_grouping(&mut self, e: &Grouping) -> T;
    fn visit_variable(&mut self, e: &Variable) -> T;
    fn visit_assignment(&mut self, e: &Assignment) -> T;
    fn visit_call(&mut self, e: &Call) -> T;
}

type RuntimeResult = Result<RuntimeValue, RuntimeError>;

pub trait Expr: dyn_clone::DynClone {
    // only valid for `Variable`. Temporary workaround for assignment parsing.
    fn var_name(&self) -> Option<Token> { None }

    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String;
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult;
}

dyn_clone::clone_trait_object!(Expr);

impl Expr for Literal {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_literal(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_literal(self)
    }
}

impl Expr for Unary {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_unary(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_unary(self)
    }
}

impl Expr for Binary {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_binary(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_binary(self)
    }
}

impl Expr for Grouping {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_grouping(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_grouping(self)
    }
}

impl Expr for Variable {
    fn var_name(&self) -> Option<Token> {
        Some(self.name.clone())
    }

    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_variable(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_variable(self)
    }
}

impl Expr for Assignment {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_assignment(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_assignment(self)
    }
}

impl Expr for Logical {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_logical(self)
    }

    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_logical(self)
    }
}

impl Expr for Call {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_call(self)
    }

    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_call(self)
    }
}