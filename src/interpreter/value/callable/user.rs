use super::super::super::Interpreter;
use super::super::super::environment::Environment;
use super::super::Value;
use super::Call;
use crate::ast::statement::FunctionDeclaration;
use crate::interpreter::RuntimeError;
use std::fmt;
use std::io;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction<'a> {
    declaration: Rc<FunctionDeclaration<'a>>,
}

impl<'a> UserFunction<'a> {
    pub fn new(declaration: Rc<FunctionDeclaration<'a>>) -> Self {
        Self { declaration }
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
        let global = Rc::clone(&interpreter.global);
        let mut environment = Environment::with_enclosing(global);
        for (parameter, argument) in self.declaration.parameters.iter().zip(arguments) {
            environment.define(parameter, argument);
        }
        let previous = Rc::clone(&interpreter.current);
        interpreter.current = environment.into_shared();
        for statement in &self.declaration.body {
            if let Err(error) = interpreter.execute(statement) {
                interpreter.current = previous;
                return Err(error);
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
