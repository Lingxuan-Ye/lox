use crate::ast::expression::{
    BinaryOperator, Expression, ExpressionIdGenerator, ExpressionKind, Literal, LogicalOperator,
    UnaryOperator,
};
use crate::ast::statement::Statement;
use crate::lexer::{LexError, Lexer};
use crate::string::LoxString;
use crate::token::{Keyword, Token, TokenKind};
use std::range::Range;

const MAX_ARITY: usize = 255;

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Result<Token, ParseError>>,
    panic_mode: bool,
    expression_id_generator: ExpressionIdGenerator,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            peeked: None,
            panic_mode: true,
            expression_id_generator: ExpressionIdGenerator::default(),
        }
    }

    pub fn source(&self) -> &'a str {
        self.lexer.source()
    }

    pub fn parse(mut self) -> Result<Vec<Statement<'a>>, Vec<ParseError>> {
        let mut statements = Vec::new();
        while let Some(result) = self.next() {
            match result {
                Err(error) => {
                    let mut errors = Vec::new();
                    errors.push(error);
                    errors.extend(self.filter_map(Result::err));
                    return Err(errors);
                }
                Ok(statement) => {
                    statements.push(statement);
                }
            }
        }
        Ok(statements)
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.peek_token() {
            match token {
                Err(_) => {
                    self.next_token();
                }
                Ok(token) => match token.kind {
                    TokenKind::Keyword(Keyword::Print)
                    | TokenKind::Keyword(Keyword::Class)
                    | TokenKind::Keyword(Keyword::Fun)
                    | TokenKind::Keyword(Keyword::Var)
                    | TokenKind::Keyword(Keyword::While)
                    | TokenKind::Keyword(Keyword::For)
                    | TokenKind::Keyword(Keyword::If)
                    | TokenKind::Keyword(Keyword::Return) => {
                        break;
                    }
                    TokenKind::Semicolon => {
                        self.next_token();
                        break;
                    }
                    _ => {
                        self.next_token();
                    }
                },
            }
        }
    }

    fn peek_token(&mut self) -> Option<&Result<Token, ParseError>> {
        if self.peeked.is_none() {
            self.peeked = self
                .lexer
                .next()
                .map(|result| result.map_err(ParseError::LexError));
        }
        self.peeked.as_ref()
    }

    fn next_token(&mut self) -> Option<Result<Token, ParseError>> {
        if self.peeked.is_none() {
            self.lexer
                .next()
                .map(|result| result.map_err(ParseError::LexError))
        } else {
            self.peeked.take()
        }
    }
}

impl<'a> Parser<'a> {
    fn declaration(&mut self) -> Result<Statement<'a>, ParseError> {
        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            if self.panic_mode {
                self.synchronize();
            } else {
                self.panic_mode = true;
            }
            return Err(error);
        };

        let result = match token.kind {
            TokenKind::Keyword(Keyword::Fun) => self.function_declaration(),
            TokenKind::Keyword(Keyword::Var) => self.variable_declaration(),
            _ => self.statement(),
        };

        if result.is_err() {
            if self.panic_mode {
                self.synchronize();
            } else {
                self.panic_mode = true;
            }
        }

        result
    }

    fn function_declaration(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::Fun) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Identifier {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let name = &self.source()[token.range];

        let token = self.next_token().require()?;
        if token.kind != TokenKind::LParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let mut parameters = Vec::new();

        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        if token.kind != TokenKind::RParen {
            let parameter_start = token.range.start;
            loop {
                let token = self.next_token().require()?;
                if token.kind != TokenKind::Identifier {
                    let error = ParseError::UnexpectedToken(token);
                    return Err(error);
                }

                if parameters.len() == MAX_ARITY {
                    self.panic_mode = false;
                    let start = parameter_start;
                    let end = token.range.end;
                    let range = Range { start, end };
                    let error = ParseError::TooManyArguments { range };
                    return Err(error);
                }

                let parameter = &self.source()[token.range];

                parameters.push(parameter);

                let Some(Ok(token)) = self.peek_token() else {
                    let Err(error) = self.next_token().require() else {
                        unreachable!()
                    };
                    return Err(error);
                };

                if token.kind != TokenKind::Comma {
                    break;
                }

                self.next_token();
            }
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::RParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::LBrace {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let mut body = Vec::new();

        loop {
            let Some(Ok(token)) = self.peek_token() else {
                let Err(error) = self.next_token().require() else {
                    unreachable!()
                };
                return Err(error);
            };

            if token.kind == TokenKind::RBrace {
                self.next_token();
                break;
            }

            let statement = self.declaration()?;
            body.push(statement);
        }

        let statement = Statement::function_declaration(name, parameters, body);
        Ok(statement)
    }

    fn variable_declaration(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::Var) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Identifier {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let name = &self.source()[token.range];

        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        let initializer = if token.kind != TokenKind::Equal {
            None
        } else {
            self.next_token();
            let expression = self.expression()?;
            Some(expression)
        };

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let statement = Statement::variable_declaration(name, initializer);
        Ok(statement)
    }

    fn statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        match token.kind {
            TokenKind::LBrace => self.block_statement(),
            TokenKind::Keyword(Keyword::While) => self.while_statement(),
            TokenKind::Keyword(Keyword::For) => self.for_statement(),
            TokenKind::Keyword(Keyword::If) => self.if_statement(),
            TokenKind::Keyword(Keyword::Print) => self.print_statement(),
            TokenKind::Keyword(Keyword::Return) => self.return_statement(),
            _ => self.expression_statement(),
        }
    }

    fn block_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::LBrace {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let mut statements = Vec::new();

        loop {
            let Some(Ok(token)) = self.peek_token() else {
                let Err(error) = self.next_token().require() else {
                    unreachable!()
                };
                return Err(error);
            };

            if token.kind == TokenKind::RBrace {
                self.next_token();
                break;
            }

            let statement = self.declaration()?;
            statements.push(statement);
        }

        let statement = Statement::block(statements);
        Ok(statement)
    }

    fn while_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::While) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::LParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let condition = self.expression()?;

        let token = self.next_token().require()?;
        if token.kind != TokenKind::RParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let body = self.statement()?;

        let statement = Statement::while_statement(condition, body);
        Ok(statement)
    }

    fn for_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::For) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::LParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        let initializer = match token.kind {
            TokenKind::Semicolon => None,
            TokenKind::Keyword(Keyword::Var) => {
                let statement = self.variable_declaration()?;
                Some(statement)
            }
            _ => {
                let statement = self.expression_statement()?;
                Some(statement)
            }
        };

        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        let condition = if token.kind == TokenKind::Semicolon {
            let range = token.range;
            let id = self.expression_id_generator.next_id();
            let literal = Literal::Boolean(true);
            Expression::literal(id, range, literal)
        } else {
            self.expression()?
        };

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        let increment = if token.kind == TokenKind::RParen {
            None
        } else {
            let expression = self.expression()?;
            Some(expression)
        };

        let token = self.next_token().require()?;
        if token.kind != TokenKind::RParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let body = self.statement()?;

        let mut statement = body;
        if let Some(increment) = increment {
            let increment = Statement::expression(increment);
            statement = Statement::block(vec![statement, increment]);
        }
        statement = Statement::while_statement(condition, statement);
        if let Some(initializer) = initializer {
            statement = Statement::block(vec![initializer, statement]);
        }
        Ok(statement)
    }

    fn if_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::If) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let token = self.next_token().require()?;
        if token.kind != TokenKind::LParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let condition = self.expression()?;

        let token = self.next_token().require()?;
        if token.kind != TokenKind::RParen {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let then_branch = self.statement()?;

        let else_branch = match self.peek_token() {
            Some(Err(_)) => {
                let Some(Err(error)) = self.next_token() else {
                    unreachable!()
                };
                return Err(error);
            }
            Some(Ok(token)) if token.kind == TokenKind::Keyword(Keyword::Else) => {
                self.next_token();
                let statement = self.statement()?;
                Some(statement)
            }
            Some(Ok(_)) | None => None,
        };

        let statement = Statement::if_statement(condition, then_branch, else_branch);
        Ok(statement)
    }

    fn print_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::Print) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let expression = self.expression()?;

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let statement = Statement::print(expression);
        Ok(statement)
    }

    fn return_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let token = self.next_token().require()?;
        if token.kind != TokenKind::Keyword(Keyword::Return) {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let keyword_range = token.range;

        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        if token.kind == TokenKind::Semicolon {
            self.next_token();
            let statement = Statement::return_statement(keyword_range, None);
            return Ok(statement);
        }

        let value = self.expression()?;

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let statement = Statement::return_statement(keyword_range, Some(value));
        Ok(statement)
    }

    fn expression_statement(&mut self) -> Result<Statement<'a>, ParseError> {
        let expression = self.expression()?;

        let token = self.next_token().require()?;
        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Err(error);
        }

        let statement = Statement::expression(expression);
        Ok(statement)
    }

    fn expression(&mut self) -> Result<Expression<'a>, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression<'a>, ParseError> {
        let lhs = self.or()?;

        let token = match self.peek_token() {
            None => return Ok(lhs),
            Some(Err(_)) => {
                let Some(Err(error)) = self.next_token() else {
                    unreachable!()
                };
                return Err(error);
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::Equal {
            return Ok(lhs);
        }

        self.next_token();

        let ExpressionKind::Variable { name } = lhs.kind else {
            self.panic_mode = false;
            let range = lhs.range;
            let error = ParseError::InvalidAssignmentTarget { range };
            return Err(error);
        };

        let value = self.assignment()?;

        let id = self.expression_id_generator.next_id();
        let start = lhs.range.start;
        let end = value.range.end;
        let range = Range { start, end };
        let expression = Expression::assignment(id, range, name, value);
        Ok(expression)
    }

    fn or(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut lhs = self.and()?;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Keyword(Keyword::Or) => LogicalOperator::Or,
                _ => break,
            };

            self.next_token();

            let rhs = self.and()?;

            let id = self.expression_id_generator.next_id();
            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::logical(id, range, operator, lhs, rhs);
        }

        Ok(lhs)
    }

    fn and(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut lhs = self.equality()?;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Keyword(Keyword::And) => LogicalOperator::And,
                _ => break,
            };

            self.next_token();

            let rhs = self.equality()?;

            let id = self.expression_id_generator.next_id();
            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::logical(id, range, operator, lhs, rhs);
        }

        Ok(lhs)
    }

    fn equality(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut lhs = self.comparison()?;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => break,
            };

            self.next_token();

            let rhs = self.comparison()?;

            let id = self.expression_id_generator.next_id();
            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(id, range, operator, lhs, rhs);
        }

        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut lhs = self.term()?;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => break,
            };

            self.next_token();

            let rhs = self.term()?;

            let id = self.expression_id_generator.next_id();
            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(id, range, operator, lhs, rhs);
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut lhs = self.factor()?;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                _ => break,
            };

            self.next_token();

            let rhs = self.factor()?;

            let id = self.expression_id_generator.next_id();
            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(id, range, operator, lhs, rhs);
        }

        Ok(lhs)
    }

    fn factor(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut lhs = self.unary()?;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                _ => break,
            };

            self.next_token();

            let rhs = self.unary()?;

            let id = self.expression_id_generator.next_id();
            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(id, range, operator, lhs, rhs);
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expression<'a>, ParseError> {
        let Some(Ok(token)) = self.peek_token() else {
            let Err(error) = self.next_token().require() else {
                unreachable!()
            };
            return Err(error);
        };

        let operator = match token.kind {
            TokenKind::Minus => UnaryOperator::Neg,
            TokenKind::Bang => UnaryOperator::Not,
            _ => return self.call(),
        };

        let start = token.range.start;

        self.next_token();

        let rhs = self.unary()?;

        let id = self.expression_id_generator.next_id();
        let end = rhs.range.end;
        let range = Range { start, end };
        let expression = Expression::unary(id, range, operator, rhs);
        Ok(expression)
    }

    fn call(&mut self) -> Result<Expression<'a>, ParseError> {
        let mut expression = self.primary()?;

        let start = expression.range.start;

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    return Err(error);
                }
                Some(Ok(token)) => token,
            };

            if token.kind != TokenKind::LParen {
                break;
            }

            self.next_token();

            let mut arguments = Vec::new();

            let Some(Ok(token)) = self.peek_token() else {
                let Err(error) = self.next_token().require() else {
                    unreachable!()
                };
                return Err(error);
            };

            if token.kind != TokenKind::RParen {
                let argument_start = token.range.start;
                loop {
                    let expression = self.expression()?;

                    if arguments.len() == MAX_ARITY {
                        self.panic_mode = false;
                        let start = argument_start;
                        let end = expression.range.end;
                        let range = Range { start, end };
                        let error = ParseError::TooManyArguments { range };
                        return Err(error);
                    }

                    arguments.push(expression);

                    let Some(Ok(token)) = self.peek_token() else {
                        let Err(error) = self.next_token().require() else {
                            unreachable!()
                        };
                        return Err(error);
                    };

                    if token.kind != TokenKind::Comma {
                        break;
                    }

                    self.next_token();
                }
            }

            let token = self.next_token().require()?;
            if token.kind != TokenKind::RParen {
                let error = ParseError::UnexpectedToken(token);
                return Err(error);
            }

            let id = self.expression_id_generator.next_id();
            let end = token.range.end;
            let range = Range { start, end };
            expression = Expression::call(id, range, expression, arguments);
        }

        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expression<'a>, ParseError> {
        let token = self.next_token().require()?;

        match token.kind {
            TokenKind::LParen => {
                let start = token.range.start;
                let expression = self.expression()?;
                let token = self.next_token().require()?;
                if token.kind != TokenKind::RParen {
                    let error = ParseError::UnexpectedToken(token);
                    return Err(error);
                }
                let id = self.expression_id_generator.next_id();
                let end = token.range.end;
                let range = Range { start, end };
                let expression = Expression::grouping(id, range, expression);
                Ok(expression)
            }

            TokenKind::Identifier => {
                let id = self.expression_id_generator.next_id();
                let range = token.range;
                let name = &self.source()[range];
                let expression = Expression::variable(id, range, name);
                Ok(expression)
            }

            TokenKind::String => {
                let start = token.range.start + 1;
                let end = token.range.end - 1;
                let raw_range = Range { start, end };
                let raw = &self.source()[raw_range];
                match LoxString::unescape(raw) {
                    Err(mut range) => {
                        range.start += raw_range.start;
                        range.end += raw_range.start;
                        let error = ParseError::InvalidEscapeSequence { range };
                        Err(error)
                    }
                    Ok(string) => {
                        let id = self.expression_id_generator.next_id();
                        let range = token.range;
                        let literal = Literal::String(string);
                        let expression = Expression::literal(id, range, literal);
                        Ok(expression)
                    }
                }
            }

            TokenKind::Number => {
                let id = self.expression_id_generator.next_id();
                let range = token.range;
                let number = self.source()[range]
                    .parse()
                    .unwrap_or_else(|_| unreachable!());
                let literal = Literal::Number(number);
                let expression = Expression::literal(id, range, literal);
                Ok(expression)
            }

            TokenKind::Keyword(Keyword::True) => {
                let id = self.expression_id_generator.next_id();
                let range = token.range;
                let literal = Literal::Boolean(true);
                let expression = Expression::literal(id, range, literal);
                Ok(expression)
            }

            TokenKind::Keyword(Keyword::False) => {
                let id = self.expression_id_generator.next_id();
                let range = token.range;
                let literal = Literal::Boolean(false);
                let expression = Expression::literal(id, range, literal);
                Ok(expression)
            }

            TokenKind::Keyword(Keyword::Nil) => {
                let id = self.expression_id_generator.next_id();
                let range = token.range;
                let literal = Literal::Nil;
                let expression = Expression::literal(id, range, literal);
                Ok(expression)
            }

            _ => {
                let error = ParseError::UnexpectedToken(token);
                Err(error)
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Statement<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.peek_token()?;
        let statement = self.declaration();
        Some(statement)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedEndOfInput,
    UnexpectedToken(Token),
    InvalidAssignmentTarget { range: Range<usize> },
    TooManyArguments { range: Range<usize> },
    InvalidEscapeSequence { range: Range<usize> },
}

trait Require {
    fn require(self) -> Result<Token, ParseError>;
}

impl Require for Option<Result<Token, ParseError>> {
    fn require(self) -> Result<Token, ParseError> {
        self.transpose()?.ok_or(ParseError::UnexpectedEndOfInput)
    }
}
