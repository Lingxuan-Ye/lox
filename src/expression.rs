use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::fmt;
use core::range::Range;

#[derive(Debug, PartialEq)]
pub struct Expression<'a> {
    pub kind: ExpressionKind<'a>,
    pub range: Range<usize>,
}

#[derive(PartialEq)]
pub enum ExpressionKind<'a> {
    Binary(Binary<'a>),
    Unary(Unary<'a>),
    Grouping(Grouping<'a>),
    Literal(Literal<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Binary<'a> {
    pub operator: BinaryOperator,
    pub lhs: Box<Expression<'a>>,
    pub rhs: Box<Expression<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq)]
pub struct Unary<'a> {
    pub operator: UnaryOperator,
    pub rhs: Box<Expression<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct Grouping<'a>(pub Box<Expression<'a>>);

#[derive(Debug, PartialEq)]
pub enum Literal<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl<'a> Expression<'a> {
    pub fn binary(operator: BinaryOperator, lhs: Self, rhs: Self, range: Range<usize>) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        let binary = Binary { operator, lhs, rhs };
        let kind = ExpressionKind::Binary(binary);
        Self { kind, range }
    }

    pub fn unary(operator: UnaryOperator, rhs: Self, range: Range<usize>) -> Self {
        let rhs = Box::new(rhs);
        let unary = Unary { operator, rhs };
        let kind = ExpressionKind::Unary(unary);
        Self { kind, range }
    }

    pub fn grouping(expression: Self, range: Range<usize>) -> Self {
        let expression = Box::new(expression);
        let grouping = Grouping(expression);
        let kind = ExpressionKind::Grouping(grouping);
        Self { kind, range }
    }

    pub fn literal(literal: Literal<'a>, range: Range<usize>) -> Self {
        let kind = ExpressionKind::Literal(literal);
        Self { kind, range }
    }
}

impl fmt::Debug for ExpressionKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionKind::Binary(binary) => binary.fmt(f),
            ExpressionKind::Unary(unary) => unary.fmt(f),
            ExpressionKind::Grouping(grouping) => grouping.fmt(f),
            ExpressionKind::Literal(literal) => literal.fmt(f),
        }
    }
}
