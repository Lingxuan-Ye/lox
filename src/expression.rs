use alloc::borrow::Cow;
use alloc::boxed::Box;

#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    Binary {
        operator: BinaryOperator,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Unary {
        operator: UnaryOperator,
        rhs: Box<Self>,
    },
    Grouping(Box<Self>),
    Literal(Literal<'a>),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    NotEqual,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, PartialEq)]
pub enum Literal<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Nil,
}
