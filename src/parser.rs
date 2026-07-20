use crate::ast::expression::{
    BinaryOperator, Expression, ExpressionKind, Literal, LogicalOperator, UnaryOperator,
};
use crate::ast::statement::Statement;
use crate::lexer::{LexError, Lexer};
use crate::string::LoxString;
use crate::token::{Keyword, Token, TokenKind};
use std::range::Range;

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Option<Result<Token, LexError>>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let lexer = Lexer::new(source);
        let peeked = None;
        Self { lexer, peeked }
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

    fn peek_token(&mut self) -> Option<&Result<Token, LexError>> {
        self.peeked
            .get_or_insert_with(|| self.lexer.next())
            .as_ref()
    }

    fn next_token(&mut self) -> Option<Result<Token, LexError>> {
        self.peeked.take().unwrap_or_else(|| self.lexer.next())
    }
}

impl<'a> Parser<'a> {
    fn declaration(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let Ok(token) = self.peek_token()? else {
            let Some(Err(error)) = self.next_token() else {
                unreachable!()
            };
            self.synchronize();
            let error = ParseError::LexError(error);
            return Some(Err(error));
        };

        let option = match token.kind {
            TokenKind::Keyword(Keyword::Var) => self.variable_declaration(),
            _ => self.statement(),
        };

        let result = option.unwrap_or_else(|| unreachable!());

        if result.is_err() {
            self.synchronize();
        }

        Some(result)
    }

    fn variable_declaration(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

        if token.kind != TokenKind::Keyword(Keyword::Var) {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        // The early returns above are unreachable when called from `Parser::declaration`.
        // They exist only for correctness should this method ever be called directly,
        // even though it is not intended to.

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::Identifier {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let name = &self.source()[token.range];

        let token = match self.peek_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(_)) => {
                let Some(Err(error)) = self.next_token() else {
                    unreachable!()
                };
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        let initializer = if token.kind != TokenKind::Equal {
            None
        } else {
            self.next_token();
            match self.expression() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(expression)) => Some(expression),
            }
        };

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let statement = Statement::variable_declaration(name, initializer);
        Some(Ok(statement))
    }

    fn statement(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let Ok(token) = self.peek_token()? else {
            let Some(Err(error)) = self.next_token() else {
                unreachable!()
            };
            let error = ParseError::LexError(error);
            return Some(Err(error));
        };

        match token.kind {
            TokenKind::LBrace => self.block_statement(),
            TokenKind::Keyword(Keyword::If) => self.if_statement(),
            TokenKind::Keyword(Keyword::While) => self.while_statement(),
            TokenKind::Keyword(Keyword::Print) => self.print_statement(),
            _ => self.expression_statement(),
        }
    }

    fn block_statement(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

        if token.kind != TokenKind::LBrace {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        // The early returns above are unreachable when called from `Parser::statement`.
        // They exist only for correctness should this method ever be called directly,
        // even though it is not intended to.

        let mut statements = Vec::new();

        loop {
            let token = match self.peek_token() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            if token.kind == TokenKind::RBrace {
                self.next_token();
                break;
            }

            match self.declaration() {
                None => unreachable!(),
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(statement)) => statements.push(statement),
            }
        }

        let statement = Statement::block(statements);
        Some(Ok(statement))
    }

    fn if_statement(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

        if token.kind != TokenKind::Keyword(Keyword::If) {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        // The early returns above are unreachable when called from `Parser::statement`.
        // They exist only for correctness should this method ever be called directly,
        // even though it is not intended to.

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::LParen {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let condition = match self.expression() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(expression)) => expression,
        };

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::RParen {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let then_branch = match self.statement() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(statement)) => statement,
        };

        match self.peek_token() {
            Some(Err(_)) => {
                let Some(Err(error)) = self.next_token() else {
                    unreachable!()
                };
                let error = ParseError::LexError(error);
                Some(Err(error))
            }
            Some(Ok(token)) if token.kind == TokenKind::Keyword(Keyword::Else) => {
                self.next_token();
                let else_branch = match self.statement() {
                    None => {
                        let error = ParseError::UnexpectedEndOfInput;
                        return Some(Err(error));
                    }
                    Some(Err(error)) => return Some(Err(error)),
                    Some(Ok(statement)) => statement,
                };
                let else_branch = Some(else_branch);
                let statement = Statement::if_statement(condition, then_branch, else_branch);
                Some(Ok(statement))
            }
            Some(Ok(_)) | None => {
                let else_branch = None;
                let statement = Statement::if_statement(condition, then_branch, else_branch);
                Some(Ok(statement))
            }
        }
    }

    fn while_statement(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

        if token.kind != TokenKind::Keyword(Keyword::While) {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        // The early returns above are unreachable when called from `Parser::statement`.
        // They exist only for correctness should this method ever be called directly,
        // even though it is not intended to.

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::LParen {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let condition = match self.expression() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(expression)) => expression,
        };

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::RParen {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let body = match self.statement() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(statement)) => statement,
        };

        let statement = Statement::while_statement(condition, body);
        Some(Ok(statement))
    }

    fn print_statement(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

        if token.kind != TokenKind::Keyword(Keyword::Print) {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        // The early returns above are unreachable when called from `Parser::statement`.
        // They exist only for correctness should this method ever be called directly,
        // even though it is not intended to.

        let expression = match self.expression() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(expression)) => expression,
        };

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let statement = Statement::print(expression);
        Some(Ok(statement))
    }

    fn expression_statement(&mut self) -> Option<Result<Statement<'a>, ParseError>> {
        let expression = match self.expression()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        let token = match self.next_token() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let statement = Statement::expression(expression);
        Some(Ok(statement))
    }

    fn expression(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let lhs = match self.or()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        let token = match self.peek_token() {
            None => return Some(Ok(lhs)),
            Some(Err(_)) => {
                let Some(Err(error)) = self.next_token() else {
                    unreachable!()
                };
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Some(Ok(token)) => token,
        };

        if token.kind != TokenKind::Equal {
            return Some(Ok(lhs));
        }

        self.next_token();

        let ExpressionKind::Variable { name } = lhs.kind else {
            let range = lhs.range;
            let error = ParseError::InvalidAssignmentTarget { range };
            return Some(Err(error));
        };

        let value = match self.assignment() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(expression)) => expression,
        };

        let start = lhs.range.start;
        let end = value.range.end;
        let range = Range { start, end };
        let expression = Expression::assignment(name, value, range);
        Some(Ok(expression))
    }

    fn or(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let mut lhs = match self.and()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Keyword(Keyword::Or) => LogicalOperator::Or,
                _ => break,
            };

            self.next_token();

            let rhs = match self.term() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => rhs,
            };

            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::logical(operator, lhs, rhs, range);
        }

        Some(Ok(lhs))
    }

    fn and(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let mut lhs = match self.equality()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Keyword(Keyword::And) => LogicalOperator::And,
                _ => break,
            };

            self.next_token();

            let rhs = match self.term() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => rhs,
            };

            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::logical(operator, lhs, rhs, range);
        }

        Some(Ok(lhs))
    }

    fn equality(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let mut lhs = match self.comparison()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => break,
            };

            self.next_token();

            let rhs = match self.comparison() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => rhs,
            };

            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(operator, lhs, rhs, range);
        }

        Some(Ok(lhs))
    }

    fn comparison(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let mut lhs = match self.term()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
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

            let rhs = match self.term() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => rhs,
            };

            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(operator, lhs, rhs, range);
        }

        Some(Ok(lhs))
    }

    fn term(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let mut lhs = match self.factor()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                _ => break,
            };

            self.next_token();

            let rhs = match self.factor() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => rhs,
            };

            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(operator, lhs, rhs, range);
        }

        Some(Ok(lhs))
    }

    fn factor(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let mut lhs = match self.unary()? {
            Err(error) => return Some(Err(error)),
            Ok(expression) => expression,
        };

        loop {
            let token = match self.peek_token() {
                None => break,
                Some(Err(_)) => {
                    let Some(Err(error)) = self.next_token() else {
                        unreachable!()
                    };
                    let error = ParseError::LexError(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                _ => break,
            };

            self.next_token();

            let rhs = match self.unary() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => rhs,
            };

            let start = lhs.range.start;
            let end = rhs.range.end;
            let range = Range { start, end };
            lhs = Expression::binary(operator, lhs, rhs, range);
        }

        Some(Ok(lhs))
    }

    fn unary(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let Ok(token) = self.peek_token()? else {
            let Some(Err(error)) = self.next_token() else {
                unreachable!()
            };
            let error = ParseError::LexError(error);
            return Some(Err(error));
        };

        let operator = match token.kind {
            TokenKind::Minus => UnaryOperator::Neg,
            TokenKind::Bang => UnaryOperator::Not,
            _ => return self.primary(),
        };

        let start = token.range.start;

        self.next_token();

        let rhs = match self.unary() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                return Some(Err(error));
            }
            Some(Err(error)) => return Some(Err(error)),
            Some(Ok(rhs)) => rhs,
        };

        let end = rhs.range.end;
        let range = Range { start, end };
        let expression = Expression::unary(operator, rhs, range);
        Some(Ok(expression))
    }

    fn primary(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::LexError(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

        // The early returns above are unreachable when called from `Parser::unary`.
        // They exist only for correctness should this method ever be called directly,
        // even though it is not intended to.

        match token.kind {
            TokenKind::LParen => {
                let start = token.range.start;

                let expression = match self.expression() {
                    None => {
                        let error = ParseError::UnexpectedEndOfInput;
                        return Some(Err(error));
                    }
                    Some(Err(error)) => return Some(Err(error)),
                    Some(Ok(expression)) => expression,
                };

                let token = match self.next_token() {
                    None => {
                        let error = ParseError::UnexpectedEndOfInput;
                        return Some(Err(error));
                    }
                    Some(Err(error)) => {
                        let error = ParseError::LexError(error);
                        return Some(Err(error));
                    }
                    Some(Ok(token)) => token,
                };

                if token.kind != TokenKind::RParen {
                    let error = ParseError::UnexpectedToken(token);
                    return Some(Err(error));
                }

                let end = token.range.end;
                let range = Range { start, end };
                let expression = Expression::grouping(expression, range);
                Some(Ok(expression))
            }

            TokenKind::Identifier => {
                let name = &self.source()[token.range];
                let range = token.range;
                let expression = Expression::variable(name, range);
                Some(Ok(expression))
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
                        Some(Err(error))
                    }
                    Ok(string) => {
                        let literal = Literal::String(string);
                        let range = token.range;
                        let expression = Expression::literal(literal, range);
                        Some(Ok(expression))
                    }
                }
            }

            TokenKind::Number => {
                let number = self.source()[token.range]
                    .parse()
                    .unwrap_or_else(|_| unreachable!());
                let literal = Literal::Number(number);
                let range = token.range;
                let expression = Expression::literal(literal, range);
                Some(Ok(expression))
            }

            TokenKind::Keyword(Keyword::True) => {
                let literal = Literal::Boolean(true);
                let range = token.range;
                let expression = Expression::literal(literal, range);
                Some(Ok(expression))
            }

            TokenKind::Keyword(Keyword::False) => {
                let literal = Literal::Boolean(false);
                let range = token.range;
                let expression = Expression::literal(literal, range);
                Some(Ok(expression))
            }

            TokenKind::Keyword(Keyword::Nil) => {
                let literal = Literal::Nil;
                let range = token.range;
                let expression = Expression::literal(literal, range);
                Some(Ok(expression))
            }

            _ => {
                let error = ParseError::UnexpectedToken(token);
                Some(Err(error))
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Statement<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.declaration()
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    LexError(LexError),
    UnexpectedEndOfInput,
    UnexpectedToken(Token),
    InvalidAssignmentTarget { range: Range<usize> },
    InvalidEscapeSequence { range: Range<usize> },
}
