use std::collections::HashMap;
use crate::RuntimeValue;

struct Environment {
    bindings: HashMap<String, RuntimeValue>,
}

impl Environment {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    fn define(&mut self, name: &str, value: &RuntimeValue) {
        self.bindings.insert(name.to_owned(), value.clone());
    }

    fn assign(&mut self, name: &str, value: &RuntimeValue) -> bool {
        match self.bindings.get_mut(name) {
            Some(entry) => {
                *entry = value.clone();
                true
            },
            None => false,
        }
    }

    fn get(&self, name: &str) -> Option<&RuntimeValue> {
        self.bindings.get(name)
    }
}

// An environment stack. We use a simple list,
// nested environments go to the back of the list.
// This means the current environment is the last
// in the list and the root environment is the fist.
pub struct EnvStack {
    envs: Vec<Environment>,
}

impl EnvStack {
    pub fn new() -> Self {
        Self {
            envs: vec![Environment::new()]
        }
    }

    pub fn pop_env(&mut self) {
        self.envs.pop();
    }

    pub fn push_env(&mut self) {
        self.envs.push(Environment::new());
    }

    pub fn define(&mut self, name: &str, value: &RuntimeValue) {
        // definition is always done in the current env.
        if let Some(current_env) = self.envs.last_mut() {
            current_env.define(name, value);
        }
    }

    pub fn assign(&mut self, name: &str, value: &RuntimeValue) -> bool {
        // assign the variable in the inner-most env
        for env in self.envs.iter_mut().rev() {
            if env.assign(name, value) {
                return true;
            }
        }

        false
    }

    pub fn get(&self, name: &str) -> Option<&RuntimeValue> {
        // return the value in the inner-most env
        for env in self.envs.iter().rev() {
            let v = env.get(name);
            if v.is_some() {
                return v;
            }
        }

        None
    }
}