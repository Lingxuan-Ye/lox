use crate::expression::{BinaryOperator, Expression, ExpressionKind, Literal, UnaryOperator};
use alloc::borrow::Cow;
use core::range::Range;

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(expression: Expression<'_>) -> Result<Value<'_>, InterpretError<'_>> {
        match expression.kind {
            ExpressionKind::Binary { operator, lhs, rhs } => {
                let lhs = Self::interpret(*lhs)?;
                let rhs = Self::interpret(*rhs)?;
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
                                range: expression.range,
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
                                range: expression.range,
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
                                range: expression.range,
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
                                range: expression.range,
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
                                range: expression.range,
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
                                range: expression.range,
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
                                range: expression.range,
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
                                range: expression.range,
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

            ExpressionKind::Unary { operator, rhs } => {
                let rhs = Self::interpret(*rhs)?;
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
                                range: expression.range,
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

            ExpressionKind::Grouping(expression) => Self::interpret(*expression),

            ExpressionKind::Literal(literal) => {
                let value = Value::from(literal);
                Ok(value)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl<'a> From<Literal<'a>> for Value<'a> {
    fn from(value: Literal<'a>) -> Self {
        match value {
            Literal::String(string) => Self::String(string),
            Literal::Number(number) => Self::Number(number),
            Literal::Boolean(boolean) => Self::Boolean(boolean),
            Literal::Nil => Self::Nil,
        }
    }
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
