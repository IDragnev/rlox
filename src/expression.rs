use std::boxed::Box;
use crate::scanner::Token;
use crate::RuntimeResult;

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
    // number of env. hops needed to find the variable
    // this expression refers to
    pub hops: Option<usize>,
}

#[derive(Clone)]
pub struct Assignment {
    pub name: Token,
    // number of env. hops needed to find the variable
    // this expression assigns to
    pub hops: Option<usize>,
    pub value: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct Call {
    pub right_paren: Token,
    pub callee: Box<dyn Expr>,
    pub args: Vec<Box<dyn Expr>>,
}

#[derive(Clone)]
pub struct Get {
    pub name: Token,
    pub object: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct Set {
    pub name: Token,
    pub object: Box<dyn Expr>,
    pub value: Box<dyn Expr>,
}

#[derive(Clone)]
pub struct This {
    pub keyword: Token,
    // number of env. hops needed to find the variable
    // this expression refers to
    pub hops: Option<usize>,
}

#[derive(Clone)]
pub struct Super {
    pub keyword: Token,
    pub method: Token,
    pub hops_to_super: Option<usize>,
    pub hops_to_this: Option<usize>,
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
    fn visit_get(&mut self, e: &Get) -> T;
    fn visit_set(&mut self, e: &Set) -> T;
    fn visit_this(&mut self, e: &This) -> T;
    fn visit_super(&mut self, e: &Super) -> T;
}

pub trait MutVisitor<T> {
    fn visit_literal(&mut self, e: &mut Literal) -> T;
    fn visit_unary(&mut self, e: &mut Unary) -> T;
    fn visit_binary(&mut self, e: &mut Binary) -> T;
    fn visit_logical(&mut self, e: &mut Logical) -> T;
    fn visit_grouping(&mut self, e: &mut Grouping) -> T;
    fn visit_variable(&mut self, e: &mut Variable) -> T;
    fn visit_assignment(&mut self, e: &mut Assignment) -> T;
    fn visit_call(&mut self, e: &mut Call) -> T;
    fn visit_get(&mut self, e: &mut Get) -> T;
    fn visit_set(&mut self, e: &mut Set) -> T;
    fn visit_this(&mut self, e: &mut This) -> T;
    fn visit_super(&mut self, e: &mut Super) -> T;
}

pub enum AssignTarget {
    Var {
        name: Token,
    },
    Get {
        object: Box<dyn Expr>,
        name: Token,
    },
}

pub trait Expr: dyn_clone::DynClone {
    // workaround for assignment parsing
    fn as_assign_target(&self) -> Option<AssignTarget> { None }

    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String;
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult;
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>);
}

dyn_clone::clone_trait_object!(Expr);

impl Expr for Literal {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_literal(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_literal(self)
    }
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
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
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
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
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
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
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
        v.visit_grouping(self)
    }
}

impl Expr for Variable {
    fn as_assign_target(&self) -> Option<AssignTarget> {
        Some(AssignTarget::Var { name: self.name.clone() })
    }

    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_variable(self)
    }
    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_variable(self)
    }
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
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
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
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
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
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
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
        v.visit_call(self)
    }
}

impl Expr for Get {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_get(self)
    }

    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_get(self)
    }
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
        v.visit_get(self)
    }
    fn as_assign_target(&self) -> Option<AssignTarget> {
        Some(AssignTarget::Get {
            name: self.name.clone(),
            object: self.object.clone(),
        })
    }
}

impl Expr for Set {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_set(self)
    }

    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_set(self)
    }
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
        v.visit_set(self)
    }
}

impl Expr for This {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_this(self)
    }

    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_this(self)
    }
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
        v.visit_this(self)
    }
}

impl Expr for Super {
    fn accept_string(&self, v: &mut dyn Visitor<String>) -> String {
        v.visit_super(self)
    }

    fn accept_rt_value(&self, v: &mut dyn Visitor<RuntimeResult>) -> RuntimeResult {
        v.visit_super(self)
    }
    fn accept_resolve(&mut self, v: &mut dyn MutVisitor<()>) {
        v.visit_super(self)
    }
}