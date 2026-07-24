use self::environment::Environment;
use self::value::{Call, Callable, NativeFunction, UserFunction, Value};
use crate::ast::expression::{
    BinaryOperator, Expression, ExpressionKind, Literal, LogicalOperator, UnaryOperator,
};
use crate::ast::statement::Statement;
use crate::lexer::LexError;
use crate::parser::{ParseError, Parser};
use crate::resolver::{Resolutions, ResolveError, Resolver};
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::io;
use std::range::Range;
use std::rc::Rc;

pub mod value;

mod environment;

#[derive(Debug)]
pub struct Interpreter<'a, W> {
    global: Rc<RefCell<Environment<'a>>>,
    current: Rc<RefCell<Environment<'a>>>,
    resolutions: Resolutions,
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
        let resolutions = Resolutions::new();
        Self {
            global,
            current,
            resolutions,
            output,
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    pub fn interpret(&mut self, source: &'a str) -> Result<(), InterpretError<'a>> {
        let parser = Parser::new(source);
        let statements = parser
            .parse()
            .map_err(|errors| InterpretError::parse(errors, source))?;

        let resolver = Resolver::new();
        self.resolutions = resolver
            .resolve(&statements)
            .map_err(|errors| InterpretError::resolve(errors, source))?;

        for statement in &statements {
            self.execute(statement)
                .map_err(|error| InterpretError::runtime(error, source))?;
        }

        Ok(())
    }

    pub fn execute(
        &mut self,
        statement: &Statement<'a>,
    ) -> Result<ControlFlow<'a>, RuntimeError<'a>> {
        match statement {
            Statement::FunctionDeclaration { declaration } => {
                let name = declaration.name.text;
                let declaration = Rc::clone(declaration);
                let closure = Rc::clone(&self.current);
                let function = UserFunction::new(declaration, closure);
                let function = Callable::User(function);
                let function = Value::Callable(function);
                self.current.borrow_mut().define(name, function);
            }

            Statement::VariableDeclaration { name, initializer } => {
                let name = name.text;
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
                let enclosing = Rc::clone(&self.current);
                self.current = Environment::with_enclosing(enclosing).into_shared();
                for statement in statements {
                    match self.execute(statement) {
                        Err(error) => {
                            self.current = previous;
                            return Err(error);
                        }
                        Ok(ControlFlow::Return(value)) => {
                            self.current = previous;
                            return Ok(ControlFlow::Return(value));
                        }
                        Ok(ControlFlow::Proceed) => (),
                    }
                }
                self.current = previous;
            }

            Statement::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    match self.execute(body)? {
                        ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                        ControlFlow::Proceed => (),
                    }
                }
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    match self.execute(then_branch)? {
                        ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                        ControlFlow::Proceed => (),
                    }
                } else if let Some(else_branch) = else_branch {
                    match self.execute(else_branch)? {
                        ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                        ControlFlow::Proceed => (),
                    }
                }
            }

            Statement::Print { expression } => {
                let value = self.evaluate(expression)?;
                writeln!(self.output, "{value}").map_err(RuntimeError::io_error)?;
            }

            Statement::Return {
                keyword_range: _,
                value,
            } => {
                let value = match value {
                    None => Value::Nil,
                    Some(value) => self.evaluate(value)?,
                };
                return Ok(ControlFlow::Return(value));
            }

            Statement::Expression { expression } => {
                self.evaluate(expression)?;
            }
        };

        Ok(ControlFlow::Proceed)
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
                if let Some(distance) = self.resolutions.get(&expression.id).copied() {
                    self.current
                        .borrow_mut()
                        .assign_at(name, value, distance)
                        .or_else(|_| unreachable!())
                } else {
                    self.global
                        .borrow_mut()
                        .assign(name, value)
                        .map_err(|_| RuntimeError::undefined_variable(range))
                }
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
                let expected = callee.arity();
                let actual = arguments.len();
                if expected != actual {
                    let error = RuntimeError::arity_mismatch(expected, actual, range);
                    return Err(error);
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.evaluate(argument))
                    .collect::<Result<Box<[_]>, _>>()?;
                callee.call(self, arguments)
            }

            ExpressionKind::Grouping { expression } => self.evaluate(expression),

            ExpressionKind::Variable { name } => {
                if let Some(distance) = self.resolutions.get(&expression.id).copied() {
                    self.current
                        .borrow()
                        .get_at(name, distance)
                        .ok_or_else(|| unreachable!())
                } else {
                    self.global
                        .borrow()
                        .get(name)
                        .ok_or(RuntimeError::undefined_variable(range))
                }
            }

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

#[derive(Debug, PartialEq)]
pub enum ControlFlow<'a> {
    Return(Value<'a>),
    Proceed,
}

#[derive(Debug)]
pub struct InterpretError<'a> {
    pub kind: InterpretErrorKind<'a>,
    pub source: &'a str,
}

#[derive(Debug)]
pub enum InterpretErrorKind<'a> {
    Parse(Vec<ParseError>),
    Resolve(Vec<ResolveError>),
    Runtime(RuntimeError<'a>),
}

impl<'a> InterpretError<'a> {
    fn parse(errors: Vec<ParseError>, source: &'a str) -> Self {
        let kind = InterpretErrorKind::Parse(errors);
        Self { kind, source }
    }

    fn resolve(errors: Vec<ResolveError>, source: &'a str) -> Self {
        let kind = InterpretErrorKind::Resolve(errors);
        Self { kind, source }
    }

    fn runtime(error: RuntimeError<'a>, source: &'a str) -> Self {
        let kind = InterpretErrorKind::Runtime(error);
        Self { kind, source }
    }
}

impl fmt::Display for InterpretError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            InterpretErrorKind::Parse(errors) => {
                for error in errors {
                    if let ParseError::LexError(error) = error {
                        f.write_str("lex error: ")?;
                        match error {
                            LexError::UnexpectedCharacter { range, char } => {
                                write!(f, "unexpected character '{char}' at {range:?}")?;
                            }
                            LexError::UnterminatedString => {
                                f.write_str("unterminated string")?;
                            }
                        }
                    } else {
                        f.write_str("parse error: ")?;
                        match error {
                            ParseError::UnexpectedEndOfInput => {
                                f.write_str("unexpected end of input")?;
                            }
                            ParseError::UnexpectedToken(token) => {
                                let range = token.range;
                                let source = &self.source[range];
                                write!(f, "unexpected token `{source}` at {range:?}")?;
                            }
                            ParseError::InvalidAssignmentTarget { range } => {
                                let source = &self.source[*range];
                                write!(f, "invalid assignment target `{source}` at {range:?}")?;
                            }
                            ParseError::TooManyArguments { range } => {
                                let source = &self.source[*range];
                                write!(
                                    f,
                                    "too many arguments at {range:?} (arguments: `{source}, ..`"
                                )?;
                            }
                            ParseError::InvalidEscapeSequence { range } => {
                                let source = &self.source[*range];
                                write!(f, "invalid escape sequence \"{source}\" at {range:?}")?;
                            }
                            _ => unreachable!(),
                        }
                    }
                    writeln!(f)?;
                }
            }

            InterpretErrorKind::Resolve(errors) => {
                for error in errors {
                    f.write_str("resolve error: ")?;
                    match error {
                        ResolveError::ReturnOutsideFunction { range } => {
                            write!(f, "return statement at {range:?} outside function")?;
                        }
                        ResolveError::ReadInOwnInitializer { range } => {
                            let source = &self.source[*range];
                            write!(
                                f,
                                "read local variable `{source}` in its own initializer at {range:?}"
                            )?;
                        }
                        ResolveError::VariableAlreadyDeclared { range } => {
                            let source = &self.source[*range];
                            write!(
                                f,
                                "variable `{source}` at {range:?} is already declared in this scope"
                            )?;
                        }
                    }
                    writeln!(f)?;
                }
            }

            InterpretErrorKind::Runtime(error) => {
                f.write_str("runtime error: ")?;
                match error {
                    RuntimeError::IoError(error) => {
                        write!(f, "{error}")?;
                    }
                    RuntimeError::InvalidBinaryOperands {
                        operator,
                        lhs,
                        rhs,
                        range,
                    } => {
                        let source = &self.source[*range];
                        write!(
                            f,
                            "invalid binary operands `{lhs}` and `{rhs}` for operator `{operator:?}` at {range:?} (evaluated from: \"{source}\")"
                        )?;
                    }
                    RuntimeError::InvalidUnaryOperand {
                        operator,
                        rhs,
                        range,
                    } => {
                        let source = &self.source[*range];
                        write!(
                            f,
                            "invalid unary operand `{rhs}` for operator `{operator:?}` at {range:?} (evaluated from: \"{source}\")"
                        )?;
                    }
                    RuntimeError::UndefinedVariable { range } => {
                        let source = &self.source[*range];
                        write!(f, "undefined variable `{source}` at {range:?}")?;
                    }
                    RuntimeError::NotCallable { value, range } => {
                        let source = &self.source[*range];
                        write!(
                            f,
                            "value `{value}` at {range:?} is not callable (evaluated from: \"{source}\")"
                        )?;
                    }
                    RuntimeError::ArityMismatch {
                        expected,
                        actual,
                        range,
                    } => {
                        let source = &self.source[*range];
                        write!(
                            f,
                            "expect {expected} argument(s), got {actual} at {range:?} (evaluated from: \"{source}\")"
                        )?;
                    }
                }
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Error for InterpretError<'_> {}

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
