
use std::boxed::Box;
use crate::scanner::Token;
use crate::eval::{
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

pub trait Visitor<T> {
    fn visit_literal(&self, v: &Box<dyn Visitor<T>>, e: &Literal) -> T;
    fn visit_unary(&self, v: &Box<dyn Visitor<T>>, e: &Unary) -> T;
    fn visit_binary(&self, v: &Box<dyn Visitor<T>>, e: &Binary) -> T;
    fn visit_ternary(&self, v: &Box<dyn Visitor<T>>, e: &Ternary) -> T;
    fn visit_grouping(&self, v: &Box<dyn Visitor<T>>, e: &Grouping) -> T;
}

pub trait Expr {
    fn accept_string(&self, v: &Box<dyn Visitor<String>>) -> String;
    fn accept_rt_value(&self, v: &Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>>) -> Result<RuntimeValue, RuntimeError>;
}

impl Expr for Literal {
    fn accept_string(&self, v: &Box<dyn Visitor<String>>) -> String {
        v.visit_literal(v, self)
    }
    fn accept_rt_value(&self, v: &Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>>) -> Result<RuntimeValue, RuntimeError> {
        v.visit_literal(v, self)
    }
}

impl Expr for Unary {
    fn accept_string(&self, v: &Box<dyn Visitor<String>>) -> String {
        v.visit_unary(v, self)
    }
    fn accept_rt_value(&self, v: &Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>>) -> Result<RuntimeValue, RuntimeError> {
        v.visit_unary(v, self)
    }
}

impl Expr for Binary {
    fn accept_string(&self, v: &Box<dyn Visitor<String>>) -> String {
        v.visit_binary(v, self)
    }
    fn accept_rt_value(&self, v: &Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>>) -> Result<RuntimeValue, RuntimeError> {
        v.visit_binary(v, self)
    }
}

impl Expr for Ternary {
    fn accept_string(&self, v: &Box<dyn Visitor<String>>) -> String {
        v.visit_ternary(v, self)
    }
    fn accept_rt_value(&self, v: &Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>>) -> Result<RuntimeValue, RuntimeError> {
        v.visit_ternary(v, self)
    }
}

impl Expr for Grouping {
    fn accept_string(&self, v: &Box<dyn Visitor<String>>) -> String {
        v.visit_grouping(v, self)
    }
    fn accept_rt_value(&self, v: &Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>>) -> Result<RuntimeValue, RuntimeError> {
        v.visit_grouping(v, self)
    }
}