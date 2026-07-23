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
            Ok(value)
        } else {
            Err(UndefinedVariable)
        }
    }

    pub(super) fn assign_at(
        &mut self,
        name: &'a str,
        value: Value<'a>,
        distance: usize,
    ) -> Result<Value<'a>, UndefinedVariable> {
        if distance == 0 {
            return self.assign(name, value);
        }

        let Some(enclosing) = &self.enclosing else {
            return Err(UndefinedVariable);
        };
        let mut environment = Rc::clone(enclosing);

        for _ in 1..distance {
            let borrow = environment.borrow();
            let Some(enclosing) = &borrow.enclosing else {
                return Err(UndefinedVariable);
            };
            let enclosing = Rc::clone(enclosing);
            drop(borrow);
            environment = enclosing;
        }

        if let Some(slot) = environment.borrow_mut().variables.get_mut(name) {
            *slot = value.clone();
            Ok(value)
        } else {
            Err(UndefinedVariable)
        }
    }

    pub(super) fn get(&self, name: &str) -> Option<Value<'a>> {
        self.variables.get(name).cloned()
    }

    pub(super) fn get_at(&self, name: &str, distance: usize) -> Option<Value<'a>> {
        if distance == 0 {
            return self.get(name);
        }

        let enclosing = self.enclosing.as_ref()?;
        let mut environment = Rc::clone(enclosing);

        for _ in 1..distance {
            let borrow = environment.borrow();
            let enclosing = borrow.enclosing.as_ref()?;
            let enclosing = Rc::clone(enclosing);
            drop(borrow);
            environment = enclosing;
        }

        environment.borrow().get(name)
    }
}

#[derive(Debug)]
pub(super) struct UndefinedVariable;
