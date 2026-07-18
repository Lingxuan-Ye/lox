use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Environment<'a> {
    variables: HashMap<&'a str, Value<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        let variables = HashMap::new();
        Self { variables }
    }

    pub fn define(&mut self, name: &'a str, value: Value<'a>) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value<'a>> {
        self.variables.get(name).cloned()
    }
}
