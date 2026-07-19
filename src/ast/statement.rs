use super::expression::Expression;
use std::range::Range;

#[derive(Debug, PartialEq)]
pub struct Statement<'a> {
    pub kind: StatementKind<'a>,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum StatementKind<'a> {
    VariableDeclaration {
        name: &'a str,
        initializer: Option<Expression<'a>>,
    },
    Block {
        statements: Vec<Statement<'a>>,
    },
    If {
        condition: Expression<'a>,
        then_branch: Box<Statement<'a>>,
        else_branch: Option<Box<Statement<'a>>>,
    },
    Print {
        expression: Expression<'a>,
    },
    Expression {
        expression: Expression<'a>,
    },
}

impl<'a> Statement<'a> {
    pub fn variable_declaration(
        name: &'a str,
        initializer: Option<Expression<'a>>,
        range: Range<usize>,
    ) -> Self {
        let kind = StatementKind::VariableDeclaration { name, initializer };
        Self { kind, range }
    }

    pub fn block(statements: Vec<Statement<'a>>, range: Range<usize>) -> Self {
        let kind = StatementKind::Block { statements };
        Self { kind, range }
    }

    pub fn if_statement(
        condition: Expression<'a>,
        then_branch: Statement<'a>,
        else_branch: Option<Statement<'a>>,
        range: Range<usize>,
    ) -> Self {
        let then_branch = Box::new(then_branch);
        let else_branch = else_branch.map(Box::new);
        let kind = StatementKind::If {
            condition,
            then_branch,
            else_branch,
        };
        Self { kind, range }
    }

    pub fn print(expression: Expression<'a>, range: Range<usize>) -> Self {
        let kind = StatementKind::Print { expression };
        Self { kind, range }
    }

    pub fn expression(expression: Expression<'a>, range: Range<usize>) -> Self {
        let kind = StatementKind::Expression { expression };
        Self { kind, range }
    }
}
