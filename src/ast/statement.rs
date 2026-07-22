use super::expression::Expression;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Statement<'a> {
    FunctionDeclaration {
        declaration: Rc<FunctionDeclaration<'a>>,
    },
    VariableDeclaration {
        name: &'a str,
        initializer: Option<Expression<'a>>,
    },
    Block {
        statements: Vec<Self>,
    },
    While {
        condition: Expression<'a>,
        body: Box<Self>,
    },
    If {
        condition: Expression<'a>,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    Print {
        expression: Expression<'a>,
    },
    Return {
        value: Option<Expression<'a>>,
    },
    Expression {
        expression: Expression<'a>,
    },
}

#[derive(Debug, PartialEq)]
pub struct FunctionDeclaration<'a> {
    pub name: &'a str,
    pub parameters: Box<[&'a str]>,
    pub body: Box<[Statement<'a>]>,
}

impl<'a> Statement<'a> {
    pub fn function_declaration(name: &'a str, parameters: Vec<&'a str>, body: Vec<Self>) -> Self {
        let parameters = parameters.into_boxed_slice();
        let body = body.into_boxed_slice();
        let declaration = FunctionDeclaration {
            name,
            parameters,
            body,
        };
        let declaration = Rc::new(declaration);
        Self::FunctionDeclaration { declaration }
    }

    pub fn variable_declaration(name: &'a str, initializer: Option<Expression<'a>>) -> Self {
        Self::VariableDeclaration { name, initializer }
    }

    pub fn block(statements: Vec<Self>) -> Self {
        Self::Block { statements }
    }

    pub fn while_statement(condition: Expression<'a>, body: Self) -> Self {
        let body = Box::new(body);
        Self::While { condition, body }
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

    pub fn print(expression: Expression<'a>) -> Self {
        Self::Print { expression }
    }

    pub fn return_statement(value: Option<Expression<'a>>) -> Self {
        Self::Return { value }
    }

    pub fn expression(expression: Expression<'a>) -> Self {
        Self::Expression { expression }
    }
}
