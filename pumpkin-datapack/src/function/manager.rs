use std::collections::HashMap;

use super::parser::MCFunction;
use crate::Identifier;

/// Manages loading, storage, and execution of `.mcfunction` functions.
#[derive(Debug, Clone)]
pub struct FunctionManager {
    /// All loaded functions by ID.
    pub functions: HashMap<Identifier, MCFunction>,
    /// Functions in the `#minecraft:tick` tag (called every tick).
    pub tick_functions: Vec<Identifier>,
    /// Functions in the `#minecraft:load` tag (called after reload).
    pub load_functions: Vec<Identifier>,
}

impl FunctionManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            tick_functions: Vec::new(),
            load_functions: Vec::new(),
        }
    }

    /// Get a function by its ID.
    #[must_use]
    pub fn get(&self, id: &Identifier) -> Option<&MCFunction> {
        self.functions.get(id)
    }

    /// Set the tick/load function lists from tag data.
    pub fn set_special_functions(
        &mut self,
        tick_functions: Vec<Identifier>,
        load_functions: Vec<Identifier>,
    ) {
        self.tick_functions = tick_functions;
        self.load_functions = load_functions;
    }

    /// Get the list of functions that run every tick.
    #[must_use]
    pub fn get_tick_functions(&self) -> &[Identifier] {
        &self.tick_functions
    }

    /// Get the list of functions that run on reload.
    #[must_use]
    pub fn get_load_functions(&self) -> &[Identifier] {
        &self.load_functions
    }

    /// Return the number of loaded functions.
    #[must_use]
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    /// Replace all state (used during reload).
    pub fn replace_with(&mut self, other: Self) {
        self.functions = other.functions;
        self.tick_functions = other.tick_functions;
        self.load_functions = other.load_functions;
    }
}

impl Default for FunctionManager {
    fn default() -> Self {
        Self::new()
    }
}
