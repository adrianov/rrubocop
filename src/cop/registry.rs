use std::collections::HashMap;

use super::Cop;

pub struct CopRegistry {
    cops: Vec<Box<dyn Cop>>,
    index: HashMap<&'static str, usize>,
}

impl CopRegistry {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            cops: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        super::layout::register_all(&mut registry);
        super::lint::register_all(&mut registry);
        super::metrics::register_all(&mut registry);
        super::naming::register_all(&mut registry);
        super::security::register_all(&mut registry);
        super::style::register_all(&mut registry);
        registry
    }

    pub fn register(&mut self, cop: Box<dyn Cop>) {
        let name = cop.name();
        let idx = self.cops.len();
        self.cops.push(cop);
        self.index.insert(name, idx);
    }

    pub fn cops(&self) -> &[Box<dyn Cop>] {
        &self.cops
    }

    pub fn get(&self, name: &str) -> Option<&dyn Cop> {
        self.index.get(name).map(|&idx| &*self.cops[idx])
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.cops.iter().map(|c| c.name()).collect()
    }

    pub fn len(&self) -> usize {
        self.cops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cops.is_empty()
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.index.get(name).copied()
    }
}
