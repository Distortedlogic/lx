use std::sync::{Arc, Mutex};

use num_bigint::BigInt;

use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub type LiveIter = Arc<Mutex<Box<dyn LxIter + Send>>>;

pub trait LxIter: Send {
    fn next_val(&mut self, span: Span) -> Result<Option<Value>, LxError>;
}

pub fn make_live(iter: impl LxIter + 'static) -> LiveIter {
    Arc::new(Mutex::new(Box::new(iter)))
}

pub fn pull_next(live: &LiveIter, span: Span) -> Result<Option<Value>, LxError> {
    live.lock()
        .map_err(|e| LxError::runtime(format!("iterator lock poisoned: {e}"), span))?
        .next_val(span)
}

pub fn collect_all(live: &LiveIter, span: Span) -> Result<Vec<Value>, LxError> {
    let mut items = Vec::new();
    loop {
        match pull_next(live, span)? {
            Some(v) => items.push(v),
            None => return Ok(items),
        }
    }
}

pub enum IterSource {
    Nat,
    Cycle(Arc<Vec<Value>>),
    Live(LiveIter),
}

impl std::fmt::Debug for IterSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterSource::Nat => write!(f, "Nat"),
            IterSource::Cycle(_) => write!(f, "Cycle(...)"),
            IterSource::Live(_) => write!(f, "Live(...)"),
        }
    }
}

impl Clone for IterSource {
    fn clone(&self) -> Self {
        match self {
            IterSource::Nat => IterSource::Nat,
            IterSource::Cycle(items) => IterSource::Cycle(items.clone()),
            IterSource::Live(live) => IterSource::Live(live.clone()),
        }
    }
}

pub fn instantiate(source: &IterSource) -> LiveIter {
    match source {
        IterSource::Nat => make_live(NatIter { current: 0 }),
        IterSource::Cycle(items) => make_live(CycleIter { items: items.clone(), index: 0 }),
        IterSource::Live(live) => live.clone(),
    }
}

pub fn from_record_next(next_fn: Value) -> LiveIter {
    make_live(RecordIter { next_fn })
}

struct NatIter {
    current: i64,
}

impl LxIter for NatIter {
    fn next_val(&mut self, _span: Span) -> Result<Option<Value>, LxError> {
        let val = Value::Int(BigInt::from(self.current));
        self.current += 1;
        Ok(Some(val))
    }
}

struct CycleIter {
    items: Arc<Vec<Value>>,
    index: usize,
}

impl LxIter for CycleIter {
    fn next_val(&mut self, span: Span) -> Result<Option<Value>, LxError> {
        if self.items.is_empty() {
            return Err(LxError::runtime("cycle: empty list", span));
        }
        let val = self.items[self.index % self.items.len()].clone();
        self.index += 1;
        Ok(Some(val))
    }
}

struct RecordIter {
    next_fn: Value,
}

impl LxIter for RecordIter {
    fn next_val(&mut self, span: Span) -> Result<Option<Value>, LxError> {
        let result = crate::builtins::call_value(&self.next_fn, Value::Unit, span)?;
        match result {
            Value::Some(v) => Ok(Some(*v)),
            Value::None => Ok(None),
            other => Err(LxError::type_err(
                format!("iterator next must return Some or None, got {}", other.type_name()),
                span,
            )),
        }
    }
}

pub struct MappedIter {
    source: LiveIter,
    func: Value,
}

impl MappedIter {
    pub fn new(source: LiveIter, func: Value) -> Self {
        Self { source, func }
    }
}

impl LxIter for MappedIter {
    fn next_val(&mut self, span: Span) -> Result<Option<Value>, LxError> {
        match pull_next(&self.source, span)? {
            Some(v) => Ok(Some(crate::builtins::call_value(&self.func, v, span)?)),
            None => Ok(None),
        }
    }
}

pub struct FilteredIter {
    source: LiveIter,
    pred: Value,
}

impl FilteredIter {
    pub fn new(source: LiveIter, pred: Value) -> Self {
        Self { source, pred }
    }
}

impl LxIter for FilteredIter {
    fn next_val(&mut self, span: Span) -> Result<Option<Value>, LxError> {
        loop {
            match pull_next(&self.source, span)? {
                Some(v) => {
                    let result = crate::builtins::call_value(&self.pred, v.clone(), span)?;
                    if result.as_bool() == Some(true) {
                        return Ok(Some(v));
                    }
                }
                None => return Ok(None),
            }
        }
    }
}
