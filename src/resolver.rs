use crate::{
    expression::{
        self,
        Expr,
    },
    statement::{
        self,
        Stmt,
    },
    scanner::Token,
};
use std::collections::HashMap;

#[derive(Copy, Clone)]
enum VarInitializerState {
    Unresolved,
    Resolved,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, VarInitializerState>>,
    errors: Vec<ResolutionError>,
}

#[derive(Debug, Clone)]
pub enum ResolutionError {
    VariableAlreadyDeclared(Token),
    CantReadLocalVarInItsInitializer(Token),
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn resolve_single_expr(&mut self, expr: &mut Box<dyn Expr>) -> Result<(), Vec<ResolutionError>> {
        self.resolve_expr(expr);

        if self.errors.len() > 0 {
            Err(self.errors.drain(..).collect())
        }
        else {
            Ok(())
        }
    }

    pub fn resolve(&mut self, stmts: &mut Vec<Box<dyn Stmt>>) -> Result<(), Vec<ResolutionError>> {
        self.resolve_stmts(stmts);

        if self.errors.len() > 0 {
            Err(self.errors.drain(..).collect())
        }
        else {
            Ok(())
        }
    }

    fn resolve_stmts(&mut self, stmts: &mut Vec<Box<dyn Stmt>>) {
        for s in stmts {
            self.resolve_stmt(s)
        }
    }

    fn resolve_stmt(&mut self, stmt: &mut Box<dyn Stmt>) {
        stmt.accept_resolve(self)
    }

    fn resolve_expr(&mut self, expr: &mut Box<dyn Expr>) {
        expr.accept_resolve(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn add_err(&mut self, e: ResolutionError) {
        self.errors.push(e);
    }

    fn declare(&mut self, name: &Token) {
        match self.scopes.last_mut() {
            Some(scope) => {
                if scope.contains_key(&name.lexeme) {
                    self.add_err(ResolutionError::VariableAlreadyDeclared(name.clone()));
                }
                else {
                    scope.insert(name.lexeme.clone(), VarInitializerState::Unresolved);
                }
            }
            None => {},
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), VarInitializerState::Resolved);
        }
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                return Some(i);
            }
        }

        None
    }

    fn resolve_function(&mut self, f: &mut statement::Function) {
        self.begin_scope();
        for p in &f.params {
            self.declare(p);
            self.define(p);
        }
        self.resolve_stmts(&mut f.body);
        self.end_scope();
     }
}

impl expression::MutVisitor<()> for Resolver {
    fn visit_variable(&mut self, e: &mut expression::Variable) {
        if let Some(scope) = self.scopes.last() {
            if let Some(VarInitializerState::Unresolved) = scope.get(&e.name.lexeme) {
                self.add_err(ResolutionError::CantReadLocalVarInItsInitializer(e.name.clone()));
                return;
            }
        }

        e.hops = self.resolve_local(&e.name);
    }

    fn visit_assignment(&mut self, e: &mut expression::Assignment) {
        self.resolve_expr(&mut e.value);
        e.hops = self.resolve_local(&e.name);
    }

    fn visit_binary(&mut self, e: &mut expression::Binary) {
        self.resolve_expr(&mut e.left);
        self.resolve_expr(&mut e.right);
    }

    fn visit_call(&mut self, e: &mut expression::Call) {
        self.resolve_expr(&mut e.callee);
        for a in &mut e.args {
            self.resolve_expr(a);
        }
    }

    fn visit_grouping(&mut self, e: &mut expression::Grouping) {
        self.resolve_expr(&mut e.0)
    }

    fn visit_logical(&mut self, e: &mut expression::Logical) {
        self.resolve_expr(&mut e.left);
        self.resolve_expr(&mut e.right);
    }

    fn visit_unary(&mut self, e: &mut expression::Unary) {
        self.resolve_expr(&mut e.right)
    }

    fn visit_literal(&mut self, _: &mut expression::Literal) {
    }

}

impl statement::MutVisitor<()> for Resolver {
    fn visit_block(&mut self, s: &mut statement::Block) {
        self.begin_scope();
        self.resolve_stmts(&mut s.statements);
        self.end_scope();
    }

    fn visit_variable(&mut self, s: &mut statement::Variable) {
        self.declare(&s.name);
        if let Some(init) = &mut s.initializer {
            self.resolve_expr(init);
        }
        self.define(&s.name);
    }

    fn visit_function(&mut self, s: &mut statement::Function) {
        self.declare(&s.name);
        self.define(&s.name);
        self.resolve_function(s)
    }

    fn visit_expr(&mut self, s: &mut statement::Expression) {
        self.resolve_expr(&mut s.expr)
    }

    fn visit_if(&mut self, s: &mut statement::If) {
        self.resolve_expr(&mut s.cond);
        self.resolve_stmt(&mut s.then_branch);

        if let Some(br) = &mut s.else_branch {
            self.resolve_stmt(br);
        }
    }

    fn visit_print(&mut self, s: &mut statement::Print) {
        self.resolve_expr(&mut s.expr)
    }

    fn visit_return(&mut self, s: &mut statement::Return) {
        if let Some(e) = &mut s.value {
            self.resolve_expr(e);
        }
    }

    fn visit_break(&mut self, _: &mut statement::Break) {
    }

    fn visit_while(&mut self, s: &mut statement::While) {
        self.resolve_expr(&mut s.cond);
        self.resolve_stmt(&mut s.body)
    }
}

