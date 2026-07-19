use std::borrow::Cow;
use std::range::Range;

#[derive(Debug, PartialEq)]
pub struct Expression<'a> {
    pub kind: ExpressionKind<'a>,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionKind<'a> {
    Assignment {
        name: &'a str,
        value: Box<Expression<'a>>,
    },
    Binary {
        operator: BinaryOperator,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    Unary {
        operator: UnaryOperator,
        rhs: Box<Expression<'a>>,
    },
    Grouping {
        expression: Box<Expression<'a>>,
    },
    Variable {
        name: &'a str,
    },
    Literal(Literal<'a>),
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

impl<'a> Expression<'a> {
    pub fn assignment(name: &'a str, value: Self, range: Range<usize>) -> Self {
        let value = Box::new(value);
        let kind = ExpressionKind::Assignment { name, value };
        Self { kind, range }
    }

    pub fn binary(operator: BinaryOperator, lhs: Self, rhs: Self, range: Range<usize>) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        let kind = ExpressionKind::Binary { operator, lhs, rhs };
        Self { kind, range }
    }

    pub fn unary(operator: UnaryOperator, rhs: Self, range: Range<usize>) -> Self {
        let rhs = Box::new(rhs);
        let kind = ExpressionKind::Unary { operator, rhs };
        Self { kind, range }
    }

    pub fn grouping(expression: Self, range: Range<usize>) -> Self {
        let expression = Box::new(expression);
        let kind = ExpressionKind::Grouping { expression };
        Self { kind, range }
    }

    pub fn variable(name: &'a str, range: Range<usize>) -> Self {
        let kind = ExpressionKind::Variable { name };
        Self { kind, range }
    }

    pub fn literal(literal: Literal<'a>, range: Range<usize>) -> Self {
        let kind = ExpressionKind::Literal(literal);
        Self { kind, range }
    }
}
