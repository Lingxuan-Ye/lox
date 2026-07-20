use super::expression::Expression;

#[derive(Debug, PartialEq)]
pub enum Statement<'a> {
    VariableDeclaration {
        name: &'a str,
        initializer: Option<Expression<'a>>,
    },
    Block {
        statements: Vec<Self>,
    },
    If {
        condition: Expression<'a>,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    While {
        condition: Expression<'a>,
        body: Box<Self>,
    },
    Print {
        expression: Expression<'a>,
    },
    Expression {
        expression: Expression<'a>,
    },
}

impl<'a> Statement<'a> {
    pub fn variable_declaration(name: &'a str, initializer: Option<Expression<'a>>) -> Self {
        Self::VariableDeclaration { name, initializer }
    }

    pub fn block(statements: Vec<Self>) -> Self {
        Self::Block { statements }
    }

    pub fn if_statement(
        condition: Expression<'a>,
        then_branch: Self,
        else_branch: Option<Self>,
    ) -> Self {
        let then_branch = Box::new(then_branch);
        let else_branch = else_branch.map(Box::new);
        Self::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    pub fn while_statement(condition: Expression<'a>, body: Self) -> Self {
        let body = Box::new(body);
        Self::While { condition, body }
    }

    pub fn print(expression: Expression<'a>) -> Self {
        Self::Print { expression }
    }

    pub fn expression(expression: Expression<'a>) -> Self {
        Self::Expression { expression }
    }
}
