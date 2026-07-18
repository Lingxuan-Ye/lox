pub use self::value::{StringValue, Value};

use self::environment::Environment;
use crate::ast::expression::{BinaryOperator, Expression, ExpressionKind, Literal, UnaryOperator};
use crate::ast::statement::{Statement, StatementKind};
use crate::parser::{ParseError, Parser};
use std::borrow::Cow;
use std::cell::RefCell;
use std::io;
use std::range::Range;
use std::rc::Rc;

mod environment;
mod value;

pub struct Interpreter<'a, W> {
    environment: Rc<RefCell<Environment<'a>>>,
    output: W,
}

impl<'a, W> Interpreter<'a, W>
where
    W: io::Write,
{
    pub fn new(output: W) -> Self {
        let environment = Environment::new().into_shared();
        Self {
            environment,
            output,
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    pub fn interpret(&mut self, source: &'a str) -> Result<(), InterpreteError<'a>> {
        let mut parser = Parser::new(source);
        let mut statements = Vec::new();

        while let Some(result) = parser.next() {
            match result {
                Err(error) => {
                    let mut errors = Vec::new();
                    errors.push(error);
                    for result in parser {
                        if let Err(error) = result {
                            errors.push(error);
                        }
                    }
                    let error = InterpreteError::Parse(errors);
                    return Err(error);
                }
                Ok(statement) => {
                    statements.push(statement);
                }
            }
        }

        for statement in statements {
            if let Err(error) = self.execute(statement) {
                let error = InterpreteError::Runtime(error);
                return Err(error);
            }
        }

        Ok(())
    }
}

impl<'a, W> Interpreter<'a, W>
where
    W: io::Write,
{
    fn execute(&mut self, statement: Statement<'a>) -> Result<(), RuntimeError<'a>> {
        let Statement { kind, .. } = statement;
        match kind {
            StatementKind::VariableDeclaration { name, initializer } => {
                self.execute_variable_declaration(name, initializer)
            }
            StatementKind::Block { statements } => self.execute_block(statements),
            StatementKind::Print { expression } => self.execute_print(expression),
            StatementKind::Expression { expression } => self.execute_expression(expression),
        }
    }

    fn execute_variable_declaration(
        &mut self,
        name: &'a str,
        initializer: Option<Expression<'a>>,
    ) -> Result<(), RuntimeError<'a>> {
        if let Some(initializer) = initializer {
            let value = self.evaluate(initializer)?;
            self.environment.borrow_mut().define(name, value);
        } else {
            let value = Value::Nil;
            self.environment.borrow_mut().define(name, value);
        }
        Ok(())
    }

    fn execute_block(&mut self, statements: Vec<Statement<'a>>) -> Result<(), RuntimeError<'a>> {
        let previous = Rc::clone(&self.environment);
        self.environment = Environment::with_enclosing(Rc::clone(&previous)).into_shared();
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                self.environment = previous;
                return Err(error);
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn execute_print(&mut self, expression: Expression<'a>) -> Result<(), RuntimeError<'a>> {
        let value = self.evaluate(expression)?;
        writeln!(self.output, "{value}").map_err(RuntimeError::IoError)
    }

    fn execute_expression(&mut self, expression: Expression<'a>) -> Result<(), RuntimeError<'a>> {
        self.evaluate(expression).map(|_| ())
    }
}

impl<'a, W> Interpreter<'a, W> {
    fn evaluate(&mut self, expression: Expression<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let Expression { kind, range } = expression;
        match kind {
            ExpressionKind::Assignment { name, value } => {
                self.evaluate_assignment(name, *value, range)
            }
            ExpressionKind::Binary { operator, lhs, rhs } => {
                self.evaluate_binary(operator, *lhs, *rhs, range)
            }
            ExpressionKind::Unary { operator, rhs } => self.evaluate_unary(operator, *rhs, range),
            ExpressionKind::Grouping { expression } => self.evaluate_grouping(*expression),
            ExpressionKind::Literal(literal) => Self::evaluate_literal(literal),
            ExpressionKind::Variable { name } => self.evaluate_variable(name, range),
        }
    }

    fn evaluate_assignment(
        &mut self,
        name: &'a str,
        value: Expression<'a>,
        range: Range<usize>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let value = self.evaluate(value)?;
        self.environment
            .borrow_mut()
            .assign(name, value)
            .map_err(|_| RuntimeError::UndefinedVariable { range })
    }

    fn evaluate_binary(
        &mut self,
        operator: BinaryOperator,
        lhs: Expression<'a>,
        rhs: Expression<'a>,
        range: Range<usize>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let lhs = self.evaluate(lhs)?;
        let rhs = self.evaluate(rhs)?;
        match operator {
            BinaryOperator::Add => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Number(lhs + rhs);
                    Ok(value)
                }
                (Value::String(lhs), Value::String(rhs)) => {
                    if Rc::ptr_eq(&lhs.0, &rhs.0) {
                        let borrow = lhs.0.borrow();
                        if borrow.is_empty() {
                            drop(borrow);
                            let value = Value::String(lhs);
                            return Ok(value);
                        }
                        let string = borrow.repeat(2);
                        let value = Value::from(string);
                        return Ok(value);
                    }
                    let lhs_count = Rc::strong_count(&lhs.0);
                    let mut lhs_borrow = lhs.0.borrow_mut();
                    let rhs_borrow = rhs.0.borrow();
                    if lhs_borrow.is_empty() {
                        drop(rhs_borrow);
                        let value = Value::String(rhs);
                        return Ok(value);
                    }
                    if rhs_borrow.is_empty() {
                        drop(lhs_borrow);
                        let value = Value::String(lhs);
                        return Ok(value);
                    }
                    if let Cow::Owned(lhs_inner) = &mut *lhs_borrow
                        && lhs_count == 1
                    {
                        lhs_inner.push_str(&rhs_borrow);
                        drop(lhs_borrow);
                        let value = Value::String(lhs);
                        return Ok(value);
                    }
                    let lhs_len = lhs_borrow.len();
                    let rhs_len = rhs_borrow.len();
                    let mut string = String::with_capacity(lhs_len + rhs_len);
                    string.push_str(&lhs_borrow);
                    string.push_str(&rhs_borrow);
                    let value = Value::from(string);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::Sub => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Number(lhs - rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::Mul => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Number(lhs * rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::Div => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Number(lhs / rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::Less => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Boolean(lhs < rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::LessEqual => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Boolean(lhs <= rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::Greater => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Boolean(lhs > rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::GreaterEqual => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Boolean(lhs >= rhs);
                    Ok(value)
                }
                (lhs, rhs) => {
                    let error = RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            BinaryOperator::Equal => {
                let value = Value::Boolean(lhs == rhs);
                Ok(value)
            }
            BinaryOperator::NotEqual => {
                let value = Value::Boolean(lhs != rhs);
                Ok(value)
            }
        }
    }

    fn evaluate_unary(
        &mut self,
        operator: UnaryOperator,
        rhs: Expression<'a>,
        range: Range<usize>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let rhs = self.evaluate(rhs)?;
        match operator {
            UnaryOperator::Neg => match rhs {
                Value::Number(number) => {
                    let value = Value::Number(-number);
                    Ok(value)
                }
                _ => {
                    let error = RuntimeError::InvalidUnaryOperand {
                        operator,
                        rhs,
                        range,
                    };
                    Err(error)
                }
            },
            UnaryOperator::Not => {
                let boolean = rhs.is_truthy();
                let value = Value::Boolean(!boolean);
                Ok(value)
            }
        }
    }

    fn evaluate_grouping(
        &mut self,
        expression: Expression<'a>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        self.evaluate(expression)
    }

    fn evaluate_literal(literal: Literal<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let value = match literal {
            Literal::String(string) => Value::from(string),
            Literal::Number(number) => Value::Number(number),
            Literal::Boolean(boolean) => Value::Boolean(boolean),
            Literal::Nil => Value::Nil,
        };
        Ok(value)
    }

    fn evaluate_variable(
        &self,
        name: &str,
        range: Range<usize>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        self.environment
            .borrow()
            .get(name)
            .ok_or(RuntimeError::UndefinedVariable { range })
    }
}

#[derive(Debug)]
pub enum InterpreteError<'a> {
    Parse(Vec<ParseError>),
    Runtime(RuntimeError<'a>),
}

#[derive(Debug)]
pub enum RuntimeError<'a> {
    IoError(io::Error),
    InvalidBinaryOperands {
        operator: BinaryOperator,
        lhs: Value<'a>,
        rhs: Value<'a>,
        range: Range<usize>,
    },
    InvalidUnaryOperand {
        operator: UnaryOperator,
        rhs: Value<'a>,
        range: Range<usize>,
    },
    UndefinedVariable {
        range: Range<usize>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() {
        let mut output = Vec::new();
        let mut interpreter = Interpreter::new(&mut output);

        let source = r#"
            var foo = 1;
            var bar = 2;
            var baz;

            foo = bar = 3;

            print foo + bar;
        "#;
        let result = interpreter.interpret(source);
        assert!(result.is_ok());
        assert_eq!(interpreter.output, b"6\n");

        let source = "foo;";
        let result = interpreter.interpret(source);
        assert!(result.is_ok());
        assert_eq!(interpreter.output, b"6\n");

        let source = "baz;";
        let result = interpreter.interpret(source);
        assert!(result.is_ok());
        assert_eq!(interpreter.output, b"6\n");

        let source = "print baz;";
        let result = interpreter.interpret(source);
        assert!(result.is_ok());
        assert_eq!(interpreter.output, b"6\nnil\n");

        let source = "qux;";
        let result = interpreter.interpret(source);
        let Err(InterpreteError::Runtime(RuntimeError::UndefinedVariable { range })) = result
        else {
            unreachable!()
        };
        assert_eq!(range, Range { start: 0, end: 3 });
        assert_eq!(interpreter.output, b"6\nnil\n");

        interpreter.output.clear();

        let source = r#"
            var a = "global a";
            var b = "global b";
            var c = "global c";
            {
                var a = "outer a";
                var b = "outer b";
                {
                    var a = "inner a";
                    print a;
                    print b;
                    print c;
                }
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        "#;
        let result = interpreter.interpret(source);
        assert!(result.is_ok());
        assert_eq!(
            interpreter.output,
            b"\
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
"
        );
    }
}
