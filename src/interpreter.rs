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

            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
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

    fn evaluate(&mut self, expression: Expression<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let Expression { kind, range } = expression;
        match kind {
            ExpressionKind::Assignment { name, value } => {
                let value = self.evaluate(*value)?;
                self.environment
                    .borrow_mut()
                    .assign(name, value)
                    .map_err(|_| RuntimeError::UndefinedVariable { range })
            }

            ExpressionKind::Binary { operator, lhs, rhs } => {
                let lhs = self.evaluate(*lhs)?;
                let rhs = self.evaluate(*rhs)?;
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

            ExpressionKind::Unary { operator, rhs } => {
                let rhs = self.evaluate(*rhs)?;
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

            ExpressionKind::Grouping { expression } => self.evaluate(*expression),

            ExpressionKind::Variable { name } => self
                .environment
                .borrow()
                .get(name)
                .ok_or(RuntimeError::UndefinedVariable { range }),

            ExpressionKind::Literal(literal) => {
                let value = match literal {
                    Literal::String(string) => Value::from(string),
                    Literal::Number(number) => Value::Number(number),
                    Literal::Boolean(boolean) => Value::Boolean(boolean),
                    Literal::Nil => Value::Nil,
                };
                Ok(value)
            }
        }
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

        let source = r"
            var foo = 1;
            var bar = 2;
            var baz;
            foo = bar = 3;
            print foo + bar;
        ";
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

        interpreter.output.clear();

        let source = r#"
            var foo = 1;
            var bar = 2;
            if (foo == bar) {
                print "foo == bar";
            } else {
                print "foo != bar";
            }
        "#;
        let result = interpreter.interpret(source);
        assert!(result.is_ok());
        assert_eq!(interpreter.output, b"foo != bar\n");
    }
}
