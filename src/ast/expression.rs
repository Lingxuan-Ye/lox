use crate::string::LoxString;
use std::range::Range;

#[derive(Debug, PartialEq)]
pub struct Expression<'a> {
    pub id: ExpressionId,
    pub range: Range<usize>,
    pub kind: ExpressionKind<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionKind<'a> {
    Logical {
        operator: LogicalOperator,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
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
    Call {
        callee: Box<Expression<'a>>,
        arguments: Vec<Expression<'a>>,
    },
    Grouping {
        expression: Box<Expression<'a>>,
    },
    Variable {
        name: &'a str,
    },
    Literal(Literal<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, PartialEq)]
pub enum Literal<'a> {
    String(LoxString<'a>),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ExpressionId(usize);

impl<'a> Expression<'a> {
    pub fn logical(
        id: ExpressionId,
        range: Range<usize>,
        operator: LogicalOperator,
        lhs: Self,
        rhs: Self,
    ) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        let kind = ExpressionKind::Logical { operator, lhs, rhs };
        Self { id, range, kind }
    }

    pub fn assignment(id: ExpressionId, range: Range<usize>, name: &'a str, value: Self) -> Self {
        let value = Box::new(value);
        let kind = ExpressionKind::Assignment { name, value };
        Self { id, range, kind }
    }

    pub fn binary(
        id: ExpressionId,
        range: Range<usize>,
        operator: BinaryOperator,
        lhs: Self,
        rhs: Self,
    ) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        let kind = ExpressionKind::Binary { operator, lhs, rhs };
        Self { id, range, kind }
    }

    pub fn unary(
        id: ExpressionId,
        range: Range<usize>,
        operator: UnaryOperator,
        rhs: Self,
    ) -> Self {
        let rhs = Box::new(rhs);
        let kind = ExpressionKind::Unary { operator, rhs };
        Self { id, range, kind }
    }

    pub fn call(id: ExpressionId, range: Range<usize>, callee: Self, arguments: Vec<Self>) -> Self {
        let callee = Box::new(callee);
        let kind = ExpressionKind::Call { callee, arguments };
        Self { id, range, kind }
    }

    pub fn grouping(id: ExpressionId, range: Range<usize>, expression: Self) -> Self {
        let expression = Box::new(expression);
        let kind = ExpressionKind::Grouping { expression };
        Self { id, range, kind }
    }

    pub fn variable(id: ExpressionId, range: Range<usize>, name: &'a str) -> Self {
        let kind = ExpressionKind::Variable { name };
        Self { id, range, kind }
    }

    pub fn literal(id: ExpressionId, range: Range<usize>, literal: Literal<'a>) -> Self {
        let kind = ExpressionKind::Literal(literal);
        Self { id, range, kind }
    }
}

#[derive(Debug, Default)]
pub struct ExpressionIdGenerator {
    next: usize,
}

impl ExpressionIdGenerator {
    pub fn next_id(&mut self) -> ExpressionId {
        let id = self.next;
        self.next += 1;
        ExpressionId(id)
    }
}
