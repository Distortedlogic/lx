use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Slot>,
    parent: Option<Arc<Env>>,
}

#[derive(Debug, Clone)]
enum Slot {
    Immutable(Value),
    Mutable(Arc<Mutex<Value>>),
}

impl Env {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<Env>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn child(self: &Arc<Self>) -> Self {
        Self::with_parent(Arc::clone(self))
    }

    pub fn bind(&mut self, name: String, value: Value) {
        self.bindings.insert(name, Slot::Immutable(value));
    }

    pub fn bind_mut(&mut self, name: String, value: Value) {
        self.bindings
            .insert(name, Slot::Mutable(Arc::new(Mutex::new(value))));
    }

    pub fn reassign(&self, name: &str, value: Value) -> Result<(), String> {
        if let Some(slot) = self.bindings.get(name) {
            match slot {
                Slot::Mutable(cell) => {
                    *cell.lock() = value;
                    return Ok(());
                }
                Slot::Immutable(_) => {
                    return Err(format!("cannot reassign immutable binding '{name}'"));
                }
            }
        }
        if let Some(parent) = &self.parent {
            return parent.reassign(name, value);
        }
        Err(format!("undefined variable '{name}'"))
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(slot) = self.bindings.get(name) {
            return Some(match slot {
                Slot::Immutable(v) => v.clone(),
                Slot::Mutable(cell) => cell.lock().clone(),
            });
        }
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }
        None
    }

    pub fn has_mut(&self, name: &str) -> bool {
        match self.bindings.get(name) {
            Some(Slot::Mutable(_)) => true,
            Some(_) => false,
            None => self.parent.as_ref().is_some_and(|p| p.has_mut(name)),
        }
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
