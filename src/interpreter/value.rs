use crate::string::LoxString;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    String(LoxString<'a>),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Value<'_> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn promote(self) -> Value<'static> {
        match self {
            Value::String(string) => {
                let string = string.into_shared();
                Value::String(string)
            }
            Value::Number(number) => Value::Number(number),
            Value::Boolean(boolean) => Value::Boolean(boolean),
            Value::Nil => Value::Nil,
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(string) => string.fmt(f),
            Value::Number(number) => number.fmt(f),
            Value::Boolean(boolean) => boolean.fmt(f),
            Value::Nil => f.write_str("nil"),
        }
    }
}
