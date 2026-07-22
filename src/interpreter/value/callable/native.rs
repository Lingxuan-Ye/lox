use super::super::super::{Interpreter, RuntimeError};
use super::super::Value;
use super::Call;
use std::fmt;
use std::io;
use std::time;

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
