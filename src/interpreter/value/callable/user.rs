use super::super::super::environment::Environment;
use super::super::super::{ControlFlow, Interpreter};
use super::super::Value;
use super::Call;
use crate::ast::statement::FunctionDeclaration;
use crate::interpreter::RuntimeError;
use std::cell::RefCell;
use std::fmt;
use std::io;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct UserFunction<'a> {
    declaration: Rc<FunctionDeclaration<'a>>,
    captured: Rc<RefCell<Environment<'a>>>,
}

impl<'a> UserFunction<'a> {
    pub(in super::super::super) fn new(
        declaration: Rc<FunctionDeclaration<'a>>,
        captured: Rc<RefCell<Environment<'a>>>,
    ) -> Self {
        Self {
            declaration,
            captured,
        }
    }

    pub fn name(&self) -> &str {
        self.declaration.name
    }

    pub fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }
}

impl<'a> Call<'a> for UserFunction<'a> {
    fn call<W>(
        &self,
        interpreter: &mut Interpreter<'a, W>,
        arguments: Box<[Value<'a>]>,
    ) -> Result<Value<'a>, RuntimeError<'a>>
    where
        W: io::Write,
    {
        let captured = Rc::clone(&self.captured);
        let mut current = Environment::with_enclosing(captured);
        for (parameter, argument) in self.declaration.parameters.iter().zip(arguments) {
            current.define(parameter, argument);
        }
        let previous = Rc::clone(&interpreter.current);
        interpreter.current = current.into_shared();
        for statement in &self.declaration.body {
            match interpreter.execute(statement) {
                Err(error) => {
                    interpreter.current = previous;
                    return Err(error);
                }
                Ok(ControlFlow::Return(value)) => {
                    interpreter.current = previous;
                    return Ok(value);
                }
                Ok(ControlFlow::Proceed) => (),
            }
        }
        interpreter.current = previous;
        Ok(Value::Nil)
    }
}

impl fmt::Display for UserFunction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        write!(f, "<fn {name}>")
    }
}

impl PartialEq for UserFunction<'_> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.declaration, &other.declaration)
            && Rc::ptr_eq(&self.captured, &other.captured)
    }
}
