use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    String(Rc<RefCell<Cow<'a, str>>>),
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
            Value::String(string) => {
                let string = string.borrow();
                write!(f, "{string}")
            }
            Value::Number(number) => {
                if number.fract() == 0.0 {
                    write!(f, "{number:.0}")
                } else {
                    write!(f, "{number}")
                }
            }
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Nil => f.write_str("nil"),
        }
    }
}
