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

impl Value<'_> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        let string = Cow::Borrowed(value);
        Value::from(string)
    }
}

impl From<String> for Value<'_> {
    fn from(value: String) -> Self {
        let string = Cow::Owned(value);
        Value::from(string)
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        let string = Rc::new(RefCell::new(value));
        Value::String(string)
    }
}

impl From<f64> for Value<'_> {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
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
