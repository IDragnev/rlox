
use std::boxed::Box;
use crate::scanner::Token;
use crate::{
    RuntimeValue,
    RuntimeError,
};

pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<dyn Expr>,
}

pub struct Binary {
    pub left: Box<dyn Expr>,
    pub right: Box<dyn Expr>,
    pub operator: Token,
}

pub struct Ternary {
    pub cond: Box<dyn Expr>,
    pub left: Box<dyn Expr>,
    pub right: Box<dyn Expr>,
}

pub struct Grouping(pub Box<dyn Expr>);

pub struct Variable {
    pub name: Token,
}

pub trait Visitor<T> {
    fn visit_literal(&self, e: &Literal) -> T;
    fn visit_unary(&self, e: &Unary) -> T;
    fn visit_binary(&self, e: &Binary) -> T;
    fn visit_ternary(&self, e: &Ternary) -> T;
    fn visit_grouping(&self, e: &Grouping) -> T;
    fn visit_variable(&self, e: &Variable) -> T;
}

type RuntimeResult = Result<RuntimeValue, RuntimeError>;

pub trait Expr {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String;
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult;
}

impl Expr for Literal {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String {
        v.visit_literal(self)
    }
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_literal(self)
    }
}

impl Expr for Unary {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String {
        v.visit_unary(self)
    }
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_unary(self)
    }
}

impl Expr for Binary {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String {
        v.visit_binary(self)
    }
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_binary(self)
    }
}

impl Expr for Ternary {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String {
        v.visit_ternary(self)
    }
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_ternary(self)
    }
}

impl Expr for Grouping {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String {
        v.visit_grouping(self)
    }
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_grouping(self)
    }
}

impl Expr for Variable {
    fn accept_string(&self, v: &dyn Visitor<String>) -> String {
        v.visit_variable(self)
    }
    fn accept_rt_value(&self, v: &dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_variable(self)
    }
}