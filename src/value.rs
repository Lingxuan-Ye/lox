use std::borrow::Cow;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl From<Value<'_>> for bool {
    fn from(value: Value<'_>) -> Self {
        match value {
            Value::Boolean(boolean) => boolean,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(string) => write!(f, "{string}"),
            Value::Number(number) => write!(f, "{number}"),
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Nil => f.write_str("nil"),
        }
    }
}
