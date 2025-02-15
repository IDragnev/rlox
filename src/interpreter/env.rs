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
            if let RuntimeValue::Callable { callable: _, closure } = value {
                if let Some(cl) = closure {
                    cl.accept(visitor)?;
                }
            }
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
        // assign the variable in the inner-most env
        match self.bindings.get_mut(name) {
            Some(entry) => {
                *entry = value.clone();
                true
            },
            None => {
                match &mut self.parent {
                    Some(p) => p.borrow_mut().assign(name, value),
                    None => false,
                }
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        // return the value in the inner-most env
        let v = self.bindings.get(name);
        match v {
            Some(inner) => Some(inner.clone()),
            None => match &self.parent {
                Some(p) => p.borrow().get(name),
                None => None,
            }
        }
    }
}