use self::environment::Environment;
use self::value::{Call, Callable, NativeFunction, UserFunction, Value};
use crate::ast::expression::{
    BinaryOperator, Expression, ExpressionKind, Literal, LogicalOperator, UnaryOperator,
};
use crate::ast::statement::Statement;
use crate::parser::{ParseError, Parser};
use std::cell::RefCell;
use std::io;
use std::range::Range;
use std::rc::Rc;

pub mod value;

mod environment;

#[derive(Debug)]
pub struct Interpreter<'a, W> {
    global: Rc<RefCell<Environment<'a>>>,
    current: Rc<RefCell<Environment<'a>>>,
    output: W,
}

impl<'a, W> Interpreter<'a, W>
where
    W: io::Write,
{
    pub fn new(output: W) -> Self {
        let mut global = Environment::new();
        for function in NativeFunction::ALL {
            let name = function.name();
            let function = Callable::Native(function);
            let function = Value::Callable(function);
            global.define(name, function);
        }
        let global = global.into_shared();
        let current = Rc::clone(&global);
        Self {
            global,
            current,
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
        match statement {
            Statement::FunctionDeclaration { declaration } => {
                let declaration = Rc::clone(declaration);
                let name = declaration.name;
                let function = UserFunction::new(declaration);
                let function = Callable::User(function);
                let function = Value::Callable(function);
                self.current.borrow_mut().define(name, function);
            }

            Statement::VariableDeclaration { name, initializer } => {
                if let Some(initializer) = initializer {
                    let value = self.evaluate(initializer)?;
                    self.current.borrow_mut().define(name, value);
                } else {
                    let value = Value::Nil;
                    self.current.borrow_mut().define(name, value);
                }
            }

            Statement::Block { statements } => {
                let previous = Rc::clone(&self.current);
                self.current = Environment::with_enclosing(Rc::clone(&previous)).into_shared();
                for statement in statements {
                    if let Err(error) = self.execute(statement) {
                        self.current = previous;
                        return Err(error);
                    }
                }
                self.current = previous;
            }

            Statement::If {
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

            Statement::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }

            Statement::Print { expression } => {
                let value = self.evaluate(expression)?;
                writeln!(self.output, "{value}").map_err(RuntimeError::io_error)?;
            }

            Statement::Expression { expression } => {
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
                self.current
                    .borrow_mut()
                    .assign(name, value)
                    .map_err(|_| RuntimeError::undefined_variable(range))
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
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::Sub => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Number(lhs - rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::Mul => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Number(lhs * rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::Div => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Number(lhs / rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::Less => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Boolean(lhs < rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::LessEqual => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Boolean(lhs <= rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::Greater => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Boolean(lhs > rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
                            Err(error)
                        }
                    },
                    BinaryOperator::GreaterEqual => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            let value = Value::Boolean(lhs >= rhs);
                            Ok(value)
                        }
                        (lhs, rhs) => {
                            let error =
                                RuntimeError::invalid_binary_operands(operator, lhs, rhs, range);
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
                            let error = RuntimeError::invalid_unary_operand(operator, rhs, range);
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

            ExpressionKind::Call { callee, arguments } => {
                let callee_range = callee.range;
                let value = self.evaluate(callee)?;
                let Value::Callable(callee) = value else {
                    let range = callee_range;
                    let error = RuntimeError::not_callable(value, range);
                    return Err(error);
                };
                let arguments = arguments
                    .iter()
                    .map(|argument| self.evaluate(argument))
                    .collect::<Result<Box<[_]>, _>>()?;
                let expected = callee.arity();
                let actual = arguments.len();
                if expected != actual {
                    let error = RuntimeError::arity_mismatch(expected, actual, range);
                    return Err(error);
                }
                callee.call(self, arguments)
            }

            ExpressionKind::Grouping { expression } => self.evaluate(expression),

            ExpressionKind::Variable { name } => self
                .current
                .borrow()
                .get(name)
                .ok_or(RuntimeError::undefined_variable(range)),

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
        lhs: Box<Value<'a>>,
        rhs: Box<Value<'a>>,
        range: Range<usize>,
    },
    InvalidUnaryOperand {
        operator: UnaryOperator,
        rhs: Box<Value<'a>>,
        range: Range<usize>,
    },
    UndefinedVariable {
        range: Range<usize>,
    },
    NotCallable {
        value: Box<Value<'a>>,
        range: Range<usize>,
    },
    ArityMismatch {
        expected: usize,
        actual: usize,
        range: Range<usize>,
    },
}

impl<'a> RuntimeError<'a> {
    fn io_error(error: io::Error) -> Self {
        Self::IoError(error)
    }

    fn invalid_binary_operands(
        operator: BinaryOperator,
        lhs: Value<'a>,
        rhs: Value<'a>,
        range: Range<usize>,
    ) -> Self {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        Self::InvalidBinaryOperands {
            operator,
            lhs,
            rhs,
            range,
        }
    }

    fn invalid_unary_operand(operator: UnaryOperator, rhs: Value<'a>, range: Range<usize>) -> Self {
        let rhs = Box::new(rhs);
        Self::InvalidUnaryOperand {
            operator,
            rhs,
            range,
        }
    }

    fn undefined_variable(range: Range<usize>) -> Self {
        Self::UndefinedVariable { range }
    }

    fn not_callable(value: Value<'a>, range: Range<usize>) -> Self {
        let value = Box::new(value);
        Self::NotCallable { value, range }
    }

    fn arity_mismatch(expected: usize, actual: usize, range: Range<usize>) -> Self {
        Self::ArityMismatch {
            expected,
            actual,
            range,
        }
    }
}
