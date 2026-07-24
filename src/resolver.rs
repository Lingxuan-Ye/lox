use crate::ast::expression::{Expression, ExpressionId, ExpressionKind};
use crate::ast::statement::Statement;
use std::collections::HashMap;
use std::range::Range;

pub type Scope<'a> = HashMap<&'a str, bool>;
pub type Resolutions = HashMap<ExpressionId, usize>;

#[derive(Debug)]
pub struct Resolver<'a> {
    scopes: Vec<Scope<'a>>,
    function_declaration_depth: usize,
    result: Result<Resolutions, Vec<ResolveError>>,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            function_declaration_depth: 0,
            result: Ok(HashMap::new()),
        }
    }

    pub fn resolve(
        mut self,
        statements: &[Statement<'a>],
    ) -> Result<Resolutions, Vec<ResolveError>> {
        for statement in statements {
            self.resolve_statement(statement);
        }
        self.result
    }

    fn resolve_statement(&mut self, statement: &Statement<'a>) {
        match statement {
            Statement::FunctionDeclaration { declaration } => {
                self.function_declaration_depth += 1;

                let range = declaration.name.range;
                let name = declaration.name.text;
                self.declare(name, range);
                self.define(name);

                let scope = HashMap::new();
                self.scopes.push(scope);

                for parameter in &declaration.parameters {
                    let range = parameter.range;
                    let parameter = parameter.text;
                    self.declare(parameter, range);
                    self.define(parameter);
                }

                for statement in &declaration.body {
                    self.resolve_statement(statement);
                }

                self.scopes.pop();

                self.function_declaration_depth -= 1;
            }

            Statement::VariableDeclaration { name, initializer } => {
                let range = name.range;
                let name = name.text;
                self.declare(name, range);
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer);
                }
                self.define(name);
            }

            Statement::Block { statements } => {
                let scope = HashMap::new();
                self.scopes.push(scope);

                for statement in statements {
                    self.resolve_statement(statement);
                }

                self.scopes.pop();
            }

            Statement::While { condition, body } => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }

            Statement::Print { expression } => {
                self.resolve_expression(expression);
            }

            Statement::Return {
                keyword_range,
                value,
            } => {
                if self.function_declaration_depth == 0 {
                    let range = *keyword_range;
                    let error = ResolveError::ReturnOutsideFunction { range };
                    match &mut self.result {
                        Err(errors) => {
                            errors.push(error);
                        }
                        Ok(_) => {
                            let errors = vec![error];
                            self.result = Err(errors);
                        }
                    }
                }
                if let Some(value) = value {
                    self.resolve_expression(value);
                }
            }

            Statement::Expression { expression } => {
                self.resolve_expression(expression);
            }
        }
    }

    fn resolve_expression(&mut self, expression: &Expression<'a>) {
        let id = expression.id;

        match &expression.kind {
            ExpressionKind::Logical {
                operator: _,
                lhs,
                rhs,
            } => {
                self.resolve_expression(lhs);
                self.resolve_expression(rhs);
            }

            ExpressionKind::Assignment { name, value } => {
                self.resolve_expression(value);
                if let Ok(resolutions) = &mut self.result {
                    for (distance, scope) in self.scopes.iter().rev().enumerate() {
                        if scope.contains_key(name) {
                            resolutions.insert(id, distance);
                            break;
                        }
                    }
                }
            }

            ExpressionKind::Binary {
                operator: _,
                lhs,
                rhs,
            } => {
                self.resolve_expression(lhs);
                self.resolve_expression(rhs);
            }

            ExpressionKind::Unary { operator: _, rhs } => {
                self.resolve_expression(rhs);
            }

            ExpressionKind::Call { callee, arguments } => {
                self.resolve_expression(callee);
                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }

            ExpressionKind::Grouping { expression } => {
                self.resolve_expression(expression);
            }

            ExpressionKind::Variable { name } => {
                if self
                    .scopes
                    .last()
                    .is_some_and(|scope| scope.get(name) == Some(&false))
                {
                    let range = expression.range;
                    let error = ResolveError::ReadInOwnInitializer { range };
                    match &mut self.result {
                        Err(errors) => {
                            errors.push(error);
                        }
                        Ok(_) => {
                            let errors = vec![error];
                            self.result = Err(errors);
                        }
                    }
                }
                if let Ok(resolutions) = &mut self.result {
                    for (distance, scope) in self.scopes.iter().rev().enumerate() {
                        if scope.contains_key(name) {
                            resolutions.insert(id, distance);
                            break;
                        }
                    }
                }
            }

            ExpressionKind::Literal(_) => (),
        }
    }

    fn declare(&mut self, name: &'a str, range: Range<usize>) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                let error = ResolveError::VariableAlreadyDeclared { range };
                match &mut self.result {
                    Err(errors) => {
                        errors.push(error);
                    }
                    Ok(_) => {
                        let errors = vec![error];
                        self.result = Err(errors);
                    }
                }
                return;
            }
            scope.insert(name, false);
        }
    }

    fn define(&mut self, name: &'a str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, true);
        }
    }
}

impl Default for Resolver<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub enum ResolveError {
    ReturnOutsideFunction { range: Range<usize> },
    ReadInOwnInitializer { range: Range<usize> },
    VariableAlreadyDeclared { range: Range<usize> },
}
