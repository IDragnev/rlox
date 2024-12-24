use std::collections::HashMap;
use crate::RuntimeValue;

pub struct Environment {
    bindings: HashMap<String, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: RuntimeValue) {
        self.bindings.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &str, value: RuntimeValue) -> bool {
        match self.bindings.get_mut(name) {
            Some(entry) => {
                *entry = value;
                true
            },
            None => false,
        }
    }

    pub fn get(&self, name: &str) -> Option<&RuntimeValue> {
        self.bindings.get(name)
    }
}