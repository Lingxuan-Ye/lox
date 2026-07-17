use crate::expression::{
    Binary, BinaryOperator, Expression, ExpressionKind, Grouping, Literal, Unary, UnaryOperator,
};
use crate::parser::{ParseError, Parser};
use crate::statement::{Statement, StatementKind};
use crate::value::Value;
use std::fmt;
use std::range::Range;

pub struct Interpreter<W> {
    output: W,
}

impl<W> Interpreter<W>
where
    W: fmt::Write,
{
    pub fn new(output: W) -> Self {
        Self { output }
    }

    pub fn interpret<'a>(&mut self, source: &'a str) -> Result<(), InterpreteError<'a>> {
        let mut parser = Parser::new(source);
        let mut errors = Vec::new();
        let mut expressions = Vec::new();

        while let Some(result) = parser.next() {
            match result {
                Err(error) => {
                    errors.push(error);
                    parser.synchronize();
                    while let Some(result) = parser.next() {
                        if let Err(error) = result {
                            errors.push(error);
                            parser.synchronize();
                        }
                    }
                }
                Ok(statement) => {
                    expressions.push(statement);
                }
            }
        }

        if !errors.is_empty() {
            let error = InterpreteError::Parse(errors);
            return Err(error);
        }

        for statement in expressions {
            if let Err(error) = self.execute(statement) {
                let error = InterpreteError::Runtime(error);
                return Err(error);
            }
        }

        Ok(())
    }
}

impl<W> Interpreter<W>
where
    W: fmt::Write,
{
    fn execute<'a>(&mut self, statement: Statement<'a>) -> Result<(), RuntimeError<'a>> {
        let Statement { kind, .. } = statement;
        match kind {
            StatementKind::Print(expression) => self.execute_print(expression),
            StatementKind::Expression(expression) => self.execute_expression(expression),
        }
    }

    fn execute_print<'a>(&mut self, expression: Expression<'a>) -> Result<(), RuntimeError<'a>> {
        let value = Self::evaluate(expression)?;
        writeln!(self.output, "{value}").map_err(RuntimeError::FmtError)
    }

    fn execute_expression<'a>(
        &mut self,
        expression: Expression<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        Self::evaluate(expression).map(|_| ())
    }
}

impl<W> Interpreter<W> {
    fn evaluate(expression: Expression<'_>) -> Result<Value<'_>, RuntimeError<'_>> {
        let Expression { kind, range } = expression;
        match kind {
            ExpressionKind::Binary(binary) => Self::evaluate_binary(binary, range),
            ExpressionKind::Unary(unary) => Self::evaluate_unary(unary, range),
            ExpressionKind::Grouping(grouping) => Self::evaluate_grouping(grouping),
            ExpressionKind::Literal(literal) => Self::evaluate_literal(literal),
        }
    }

    fn evaluate_binary(
        binary: Binary<'_>,
        range: Range<usize>,
    ) -> Result<Value<'_>, RuntimeError<'_>> {
        let Binary { operator, lhs, rhs } = binary;
        let lhs = Self::evaluate(*lhs)?;
        let rhs = Self::evaluate(*rhs)?;
        match operator {
            BinaryOperator::Add => match (lhs, rhs) {
                (Value::Number(lhs), Value::Number(rhs)) => {
                    let value = Value::Number(lhs + rhs);
                    Ok(value)
                }
                (Value::String(mut lhs), Value::String(rhs)) => {
                    lhs += rhs;
                    let value = Value::String(lhs);
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
        unary: Unary<'_>,
        range: Range<usize>,
    ) -> Result<Value<'_>, RuntimeError<'_>> {
        let Unary { operator, rhs } = unary;
        let rhs = Self::evaluate(*rhs)?;
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
                let boolean = bool::from(rhs);
                let value = Value::Boolean(!boolean);
                Ok(value)
            }
        }
    }

    fn evaluate_grouping(grouping: Grouping<'_>) -> Result<Value<'_>, RuntimeError<'_>> {
        let Grouping(expression) = grouping;
        Self::evaluate(*expression)
    }

    fn evaluate_literal(literal: Literal<'_>) -> Result<Value<'_>, RuntimeError<'_>> {
        let value = match literal {
            Literal::String(string) => Value::String(string),
            Literal::Number(number) => Value::Number(number),
            Literal::Boolean(boolean) => Value::Boolean(boolean),
            Literal::Nil => Value::Nil,
        };
        Ok(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpreteError<'a> {
    Parse(Vec<ParseError<'a>>),
    Runtime(RuntimeError<'a>),
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError<'a> {
    FmtError(fmt::Error),
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
}
