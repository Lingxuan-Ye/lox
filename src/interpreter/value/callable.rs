use super::super::environment::Environment;
use super::super::{ControlFlow, Interpreter, RuntimeError};
use super::Value;
use crate::ast::statement::FunctionDeclaration;
use std::cell::RefCell;
use std::fmt;
use std::io;
use std::rc::Rc;
use std::time;

pub trait Call<'a> {
    fn call<W>(
        &self,
        interpreter: &mut Interpreter<'a, W>,
        arguments: Box<[Value<'a>]>,
    ) -> Result<Value<'a>, RuntimeError<'a>>
    where
        W: io::Write;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Callable<'a> {
    User(UserFunction<'a>),
    Native(NativeFunction),
}

impl Callable<'_> {
    pub fn name(&self) -> &str {
        match self {
            Self::User(function) => function.name(),
            Self::Native(function) => function.name(),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Self::User(function) => function.arity(),
            Self::Native(function) => function.arity(),
        }
    }
}

impl<'a> Call<'a> for Callable<'a> {
    fn call<W>(
        &self,
        interpreter: &mut Interpreter<'a, W>,
        arguments: Box<[Value<'a>]>,
    ) -> Result<Value<'a>, RuntimeError<'a>>
    where
        W: io::Write,
    {
        match self {
            Self::User(function) => function.call(interpreter, arguments),
            Self::Native(function) => function.call(interpreter, arguments),
        }
    }
}

impl fmt::Display for Callable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User(function) => function.fmt(f),
            Self::Native(function) => function.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction<'a> {
    declaration: Rc<FunctionDeclaration<'a>>,
    captured: Rc<RefCell<Environment<'a>>>,
}

impl<'a> UserFunction<'a> {
    pub(in super::super) fn new(
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

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunction {
    Clock,
}

impl NativeFunction {
    pub const ALL: [Self; 1] = [Self::Clock];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Clock => Clock::NAME,
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Self::Clock => Clock::ARITY,
        }
    }
}

impl<'a> Call<'a> for NativeFunction {
    fn call<W>(
        &self,
        interpreter: &mut Interpreter<'a, W>,
        arguments: Box<[Value<'a>]>,
    ) -> Result<Value<'a>, RuntimeError<'a>>
    where
        W: io::Write,
    {
        match self {
            Self::Clock => Clock.call(interpreter, arguments),
        }
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        write!(f, "<native fn {name}>")
    }
}

#[derive(Debug)]
struct Clock;

impl Clock {
    const NAME: &str = "clock";
    const ARITY: usize = 0;
}

impl<'a> Call<'a> for Clock {
    fn call<W>(
        &self,
        _interpreter: &mut Interpreter<'a, W>,
        _arguments: Box<[Value<'a>]>,
    ) -> Result<Value<'a>, RuntimeError<'a>>
    where
        W: io::Write,
    {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(Value::Number(now))
    }
}
