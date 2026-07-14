use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::range::Range;

#[derive(Debug, PartialEq)]
pub struct Expression<'a> {
    pub kind: ExpressionKind<'a>,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionKind<'a> {
    Binary {
        operator: BinaryOperator,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    Unary {
        operator: UnaryOperator,
        rhs: Box<Expression<'a>>,
    },
    Grouping(Box<Expression<'a>>),
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
