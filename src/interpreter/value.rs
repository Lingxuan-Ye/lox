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
            Self::Boolean(boolean) => *boolean,
            Self::Nil => false,
            _ => true,
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(string) => string.fmt(f),
            Self::Number(number) => number.fmt(f),
            Self::Boolean(boolean) => boolean.fmt(f),
            Self::Nil => f.write_str("nil"),
        }
    }
}
