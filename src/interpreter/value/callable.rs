pub use self::native::NativeFunction;
pub use self::user::UserFunction;

use super::super::{Interpreter, RuntimeError};
use super::Value;
use std::fmt;
use std::io;

mod native;
mod user;

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
    Native(NativeFunction),
    User(UserFunction<'a>),
}

impl Callable<'_> {
    pub fn name(&self) -> &str {
        match self {
            Self::Native(function) => function.name(),
            Self::User(function) => function.name(),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Self::Native(function) => function.arity(),
            Self::User(function) => function.arity(),
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
            Self::Native(function) => function.call(interpreter, arguments),
            Self::User(function) => function.call(interpreter, arguments),
        }
    }
}

impl fmt::Display for Callable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(function) => function.fmt(f),
            Self::User(function) => function.fmt(f),
        }
    }
}
