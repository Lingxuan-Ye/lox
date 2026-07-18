use super::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub(super) struct Environment<'a> {
    pub(super) enclosing: Option<Rc<RefCell<Self>>>,
    variables: HashMap<&'a str, Value<'a>>,
}

impl<'a> Environment<'a> {
    pub(super) fn new() -> Self {
        let enclosing = None;
        let variables = HashMap::new();
        Self {
            enclosing,
            variables,
        }
    }

    pub(super) fn with_enclosing(enclosing: Rc<RefCell<Self>>) -> Self {
        let enclosing = Some(enclosing);
        let variables = HashMap::new();
        Self {
            enclosing,
            variables,
        }
    }

    pub(super) fn into_shared(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub(super) fn define(&mut self, name: &'a str, value: Value<'a>) {
        self.variables.insert(name, value);
    }

    pub(super) fn assign(
        &mut self,
        name: &'a str,
        value: Value<'a>,
    ) -> Result<Value<'a>, UndefinedVariable> {
        if let Some(slot) = self.variables.get_mut(name) {
            *slot = value.clone();
            return Ok(value);
        }
        if let Some(enclosing) = self.enclosing.as_deref() {
            return enclosing.borrow_mut().assign(name, value);
        }
        Err(UndefinedVariable)
    }

    pub(super) fn get(&self, name: &str) -> Option<Value<'a>> {
        if let Some(value) = self.variables.get(name).cloned() {
            return Some(value);
        }
        if let Some(enclosing) = self.enclosing.as_deref() {
            return enclosing.borrow().get(name);
        }
        None
    }
}

#[derive(Debug, Default)]
pub(super) struct UndefinedVariable;
