use crate::{
    expression::{
        self,
        Expr,
    },
    statement::{
        self,
        Stmt,
    },
    scanner::{
        TokenType,
        Token,
    },
};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq)]
enum VarInitializerState {
    Unresolved,
    Resolved,
}

#[derive(Clone)]
struct LocalVarState {
    var_name: Token,
    init_state: VarInitializerState,
    used: bool,
}

#[derive(Copy, Clone, PartialEq)]
enum Context {
    Function,
    Method,
    InitializerMethod,
    Loop,
    Class,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, LocalVarState>>,
    errors: Vec<ResolutionError>,
    warnings: Vec<Warning>,
    context: Vec<Context>,
}

#[derive(Debug, Clone)]
pub enum ResolutionError {
    VariableAlreadyDeclared(Token),
    CantReadLocalVarInItsInitializer(Token),
    ReturnNotInFunction(Token),
    CantReturnValueFromAnInitializer(Token),
    BreakNotInLoop(Token),
    ThisNotInsideClass(Token),
    ClassCantInheritFromItself(Token),
}

#[derive(Debug, Clone)]
pub enum Warning {
    UnusedLocalVar(Token),
}

pub struct ResolutionResult {
    pub warnings: Option<Vec<Warning>>,
    pub errors: Option<Vec<ResolutionError>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            context: Vec::new(),
        }
    }

    pub fn resolve_single_expr(&mut self, expr: &mut Box<dyn Expr>) -> Result<(), Vec<ResolutionError>> {
        self.resolve_expr(expr);
        self.warnings.clear();

        if self.errors.len() > 0 {
            Err(self.errors.drain(..).collect())
        }
        else {
            Ok(())
        }
    }

    pub fn resolve(&mut self, stmts: &mut Vec<Box<dyn Stmt>>) -> ResolutionResult {
        self.resolve_stmts(stmts);

        let mut result = ResolutionResult {
            warnings: None,
            errors: None,
        };

        if self.warnings.len() > 0 {
            result.warnings = Some(
                self.warnings.drain(..).collect(),
            );
        }
        if self.errors.len() > 0 {
            result.errors = Some(
                self.errors.drain(..).collect(),
            );
        }

        result
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
        self.check_for_unused_locals();
        self.scopes.pop();
    }

    fn check_for_unused_locals(&mut self) {
        if let Some(scope) = self.scopes.last() {
            for local_var in scope.iter() {
                if local_var.1.used == false {
                    self.warnings.push(
                        Warning::UnusedLocalVar(local_var.1.var_name.clone())
                    );
                }
            }
        }
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
                    scope.insert(
                        name.lexeme.clone(),
                        LocalVarState {
                            var_name: name.clone(),
                            init_state: VarInitializerState::Unresolved,
                            used: false
                        }
                    );
                }
            }
            None => {},
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.lexeme.clone(),
                LocalVarState {
                    var_name: name.clone(),
                    init_state: VarInitializerState::Resolved,
                    used: false
                }
            );
        }
    }

    fn define_this(&mut self) {
        let name = Token {
            token_type: TokenType::This,
            lexeme: "this".to_owned(),
            literal: None,
            line: 0,
            column: 0,
        };

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.lexeme.clone(),
                LocalVarState {
                    var_name: name.clone(),
                    init_state: VarInitializerState::Resolved,
                    used: true, // must not emit a warning
                }
            );
        }
    }

    fn define_super(&mut self) {
        let name = Token {
            token_type: TokenType::Super,
            lexeme: "super".to_owned(),
            literal: None,
            line: 0,
            column: 0,
        };

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.lexeme.clone(),
                LocalVarState {
                    var_name: name.clone(),
                    init_state: VarInitializerState::Resolved,
                    used: true, // must not emit a warning
                }
            );
        }
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for (i, scope) in self.scopes.iter_mut().rev().enumerate() {
            match scope.get_mut(&name.lexeme) {
                Some(var_state) => {
                    var_state.used = true;
                    return Some(i);
                },
                None => {},
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
            if let Some(var_state) = scope.get(&e.name.lexeme) {
                if var_state.init_state == VarInitializerState::Unresolved {
                    self.add_err(ResolutionError::CantReadLocalVarInItsInitializer(e.name.clone()));
                    return;
                }
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

    fn visit_get(&mut self, e: &mut expression::Get) {
        self.resolve_expr(&mut e.object)
    }

    fn visit_set(&mut self, e: &mut expression::Set) {
        self.resolve_expr(&mut e.object);
        self.resolve_expr(&mut e.value);
    }

    fn visit_this(&mut self, e: &mut expression::This) {
        if let None = self.context.iter().find(|&c| *c == Context::Class) {
            self.add_err(ResolutionError::ThisNotInsideClass(e.keyword.clone()));
            return;
        }

        e.hops = self.resolve_local(&e.keyword);
    }

    fn visit_super(&mut self, e: &mut expression::Super) {
        e.hops_to_super = self.resolve_local(&e.keyword);
        e.hops_to_this = self.resolve_local(&Token {
            token_type: TokenType::This,
            lexeme: "this".to_owned(),
            literal: None,
            line: 0,
            column: 0
        });
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
        self.context.push(Context::Function);

        self.declare(&s.name);
        self.define(&s.name);
        self.resolve_function(s);

        self.context.pop();
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
        let iter_fun_context = self.context
            .iter()
            .copied()
            .rev()
            .find(|&c| c == Context::Function || c == Context::Method || c == Context::InitializerMethod);
        let inside_fun = iter_fun_context.is_some();
        if inside_fun == false {
            self.add_err(ResolutionError::ReturnNotInFunction(s.keyword.clone()));
            return;
        }

        if let Some(Context::InitializerMethod) = iter_fun_context {
            if s.value.is_some() {
                self.add_err(ResolutionError::CantReturnValueFromAnInitializer(s.keyword.clone()));
                return;
            }
        }

        if let Some(e) = &mut s.value {
            self.resolve_expr(e);
        }
    }

    fn visit_break(&mut self, s: &mut statement::Break) {
        for c in self.context.iter().copied().rev() {
            match c {
                Context::Function | Context::Method | Context::InitializerMethod => {
                    self.add_err(ResolutionError::BreakNotInLoop(s.keyword.clone()));
                    return;
                },
                Context::Loop => {
                    return;
                },
                Context::Class => { },
            }
        }

        self.add_err(ResolutionError::BreakNotInLoop(s.keyword.clone()));
    }

    fn visit_while(&mut self, s: &mut statement::While) {
        self.context.push(Context::Loop);

        self.resolve_expr(&mut s.cond);
        self.resolve_stmt(&mut s.body);

        self.context.pop();
    }

    fn visit_class(&mut self, s: &mut statement::Class) {
        self.context.push(Context::Class);

        // allow storing a class as a local variable
        self.declare(&s.name);
        self.define(&s.name);

        if let Some(sup) = &mut s.super_class {
            if sup.name.lexeme == s.name.lexeme {
                self.add_err(ResolutionError::ClassCantInheritFromItself(sup.name.clone()));
                return;
            }

            sup.hops = self.resolve_local(&sup.name);
        }

        if s.super_class.is_some() {
            self.begin_scope(); // super
            self.define_super();
        }

        self.begin_scope(); // this
        self.define_this();

        for m in &mut s.methods {
            let method_context = 
                if m.name.lexeme != "init" {
                    Context::Method
                }
                else {
                    Context::InitializerMethod
                };
            self.context.push(method_context);

            // self.declare(&s.name);
            // self.define(&s.name);
            self.resolve_function(m);

            self.context.pop(); // method
        }

        self.end_scope(); // this

        if s.super_class.is_some() {
            self.end_scope(); // super
        }

        self.context.pop(); // class
    }
}

