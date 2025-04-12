use std::{
    collections::HashMap,
    cell::RefCell,
};
use crate::RuntimeValue;
use dumpster::{
    Trace,
    unsync::Gc,
    Visitor,
};

#[derive(Clone)]
pub struct Environment {
    parent: Option<Gc<RefCell<Environment>>>,
    bindings: HashMap<String, RuntimeValue>,
}

unsafe impl Trace for Environment {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        if let Some(p) = &self.parent {
            p.accept(visitor)?;
        }

        for (_, value) in &self.bindings {
            value.accept(visitor)?;
        }

        Ok(())
    }
}

impl Environment {
    pub fn root() -> Self {
        Self {
            parent: None,
            bindings: HashMap::new(),
        }
    }

    pub fn child(parent_env: Gc<RefCell<Environment>>) -> Self {
        Self {
            parent: Some(parent_env),
            bindings: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: &RuntimeValue) {
        // definition is always done in the current env
        self.bindings.insert(name.to_owned(), value.clone());
    }

    pub fn assign(&mut self, name: &str, value: &RuntimeValue) -> bool {
        match self.bindings.get_mut(name) {
            Some(entry) => {
                *entry = value.clone();
                true
            },
            None => false,
        }
    }

    pub fn assign_at(&mut self, name: &str, value: &RuntimeValue,  hops: usize) -> bool {
        if hops == 0 {
            self.assign(name, value)
        }
        else {
            match &mut self.parent {
                Some(p) => p.borrow_mut().assign_at(name, value, hops - 1),
                None => false,
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        self.bindings.get(name).map(|v| v.clone())
    }

    pub fn get_at(&self, name: &str, hops: usize) -> Option<RuntimeValue> {
        if hops == 0 {
            self.get(name)
        }
        else {
            match &self.parent {
                Some(p) => p.borrow().get_at(name, hops - 1),
                None => None,
            }
        }
    }
}