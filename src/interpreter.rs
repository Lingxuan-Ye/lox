pub use self::value::Value;

use self::environment::Environment;
use crate::ast::expression::{
    BinaryOperator, Expression, ExpressionKind, Literal, LogicalOperator, UnaryOperator,
};
use crate::ast::statement::{Statement, StatementKind};
use crate::parser::{ParseError, Parser};
use std::cell::RefCell;
use std::io;
use std::range::Range;
use std::rc::Rc;

mod environment;
mod value;

#[derive(Debug)]
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

    pub fn interpret(&mut self, source: &'a str) -> Result<(), InterpretError<'a>> {
        let parser = Parser::new(source);
        let statements = parser.parse().map_err(InterpretError::Parse)?;
        for statement in &statements {
            self.execute(statement).map_err(InterpretError::Runtime)?;
        }
        Ok(())
    }

    pub fn execute(&mut self, statement: &Statement<'a>) -> Result<(), RuntimeError<'a>> {
        match &statement.kind {
            StatementKind::VariableDeclaration { name, initializer } => {
                if let Some(initializer) = initializer {
                    let value = self.evaluate(initializer)?;
                    self.environment.borrow_mut().define(name, value);
                } else {
                    let value = Value::Nil;
                    self.environment.borrow_mut().define(name, value);
                }
            }

            StatementKind::Block { statements } => {
                let previous = Rc::clone(&self.environment);
                self.environment = Environment::with_enclosing(Rc::clone(&previous)).into_shared();
                for statement in statements {
                    if let Err(error) = self.execute(statement) {
                        self.environment = previous;
                        return Err(error);
                    }
                }
                self.environment = previous;
            }

            StatementKind::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }

            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }

            StatementKind::Print { expression } => {
                let value = self.evaluate(expression)?;
                writeln!(self.output, "{value}").map_err(RuntimeError::IoError)?;
            }

            StatementKind::Expression { expression } => {
                self.evaluate(expression)?;
            }
        };
        Ok(())
    }

    pub fn evaluate(&mut self, expression: &Expression<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let range = expression.range;
        match &expression.kind {
            ExpressionKind::Logical { operator, lhs, rhs } => {
                let lhs = self.evaluate(lhs)?;
                match operator {
                    LogicalOperator::And => {
                        if !lhs.is_truthy() {
                            return Ok(lhs);
                        }
                    }
                    LogicalOperator::Or => {
                        if lhs.is_truthy() {
                            return Ok(lhs);
                        }
                    }
                }
                let rhs = self.evaluate(rhs)?;
                Ok(rhs)
            }

            ExpressionKind::Assignment { name, value } => {
                let value = self.evaluate(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name, value)
                    .map_err(|_| RuntimeError::UndefinedVariable { range })
            }

            ExpressionKind::Binary { operator, lhs, rhs } => {
                let lhs = self.evaluate(lhs)?;
                let rhs = self.evaluate(rhs)?;
                let operator = *operator;
                match operator {
                    BinaryOperator::Add => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Number(lhs + rhs);
                            Ok(value)
                        }
                        (Value::String(lhs), Value::String(rhs)) => {
                            let value = Value::String(lhs + rhs);
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

            ExpressionKind::Unary { operator, rhs } => {
                let rhs = self.evaluate(rhs)?;
                let operator = *operator;
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

            ExpressionKind::Grouping { expression } => self.evaluate(expression),

            ExpressionKind::Variable { name } => self
                .environment
                .borrow()
                .get(name)
                .ok_or(RuntimeError::UndefinedVariable { range }),

            ExpressionKind::Literal(literal) => {
                let value = match literal {
                    Literal::String(string) => Value::String(string.clone()),
                    Literal::Number(number) => Value::Number(*number),
                    Literal::Boolean(boolean) => Value::Boolean(*boolean),
                    Literal::Nil => Value::Nil,
                };
                Ok(value)
            }
        }
    }
}

#[derive(Debug)]
pub enum InterpretError<'a> {
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
