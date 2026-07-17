use crate::expression::Expression;
use std::range::Range;

#[derive(Debug, PartialEq)]
pub struct Statement<'a> {
    pub kind: StatementKind<'a>,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum StatementKind<'a> {
    Print(Expression<'a>),
    Expression(Expression<'a>),
}

impl<'a> Statement<'a> {
    pub fn print(expression: Expression<'a>, range: Range<usize>) -> Self {
        let kind = StatementKind::Print(expression);
        Self { kind, range }
    }

    pub fn expression(expression: Expression<'a>, range: Range<usize>) -> Self {
        let kind = StatementKind::Expression(expression);
        Self { kind, range }
    }
}
