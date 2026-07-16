use crate::expression::{
    Binary, BinaryOperator, Expression, ExpressionKind, Grouping, Literal, Unary, UnaryOperator,
};
use alloc::borrow::Cow;
use core::range::Range;

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(expression: Expression<'_>) -> Result<Value<'_>, InterpretError<'_>> {
        Self::evaluate(expression)
    }

    fn evaluate(expression: Expression<'_>) -> Result<Value<'_>, InterpretError<'_>> {
        let range = expression.range;
        match expression.kind {
            ExpressionKind::Binary(binary) => Self::evaluate_binary(binary, range),
            ExpressionKind::Unary(unary) => Self::evaluate_unary(unary, range),
            ExpressionKind::Grouping(grouping) => Self::evaluate_grouping(grouping, range),
            ExpressionKind::Literal(literal) => Self::evaluate_literal(literal, range),
        }
    }

    fn evaluate_binary(
        binary: Binary<'_>,
        range: Range<usize>,
    ) -> Result<Value<'_>, InterpretError<'_>> {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
                    let error = InterpretError::InvalidBinaryOperands {
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
    ) -> Result<Value<'_>, InterpretError<'_>> {
        let Unary { operator, rhs } = unary;
        let rhs = Self::evaluate(*rhs)?;
        match operator {
            UnaryOperator::Neg => match rhs {
                Value::Number(number) => {
                    let value = Value::Number(-number);
                    Ok(value)
                }
                _ => {
                    let error = InterpretError::InvalidUnaryOperand {
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

    fn evaluate_grouping(
        grouping: Grouping<'_>,
        _range: Range<usize>,
    ) -> Result<Value<'_>, InterpretError<'_>> {
        let Grouping(expression) = grouping;
        Self::evaluate(*expression)
    }

    fn evaluate_literal(
        literal: Literal<'_>,
        _range: Range<usize>,
    ) -> Result<Value<'_>, InterpretError<'_>> {
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
pub enum Value<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl From<Value<'_>> for bool {
    fn from(value: Value<'_>) -> Self {
        match value {
            Value::Boolean(boolean) => boolean,
            Value::Nil => false,
            _ => true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpretError<'a> {
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
