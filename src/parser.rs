use crate::ast::expression::{BinaryOperator, Expression, ExpressionKind, Literal, UnaryOperator};
use crate::ast::statement::Statement;
use crate::lexer::{LexError, Lexer};
use crate::token::{Keyword, Token, TokenKind};
use std::borrow::Cow;
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

        let Some(result) = option else { unreachable!() };

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

        let start = token.range.start;

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

        let end = token.range.end;
        let range = Range { start, end };
        let statement = Statement::variable_declaration(name, initializer, range);
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

        let start = token.range.start;
        let end;

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
                end = token.range.end;
                self.next_token();
                break;
            }

            match self.declaration() {
                None => unreachable!(),
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(statement)) => statements.push(statement),
            }
        }

        let range = Range { start, end };
        let statement = Statement::block(statements, range);
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

        if token.kind != TokenKind::Semicolon {
            let error = ParseError::UnexpectedToken(token);
            return Some(Err(error));
        }

        let end = token.range.end;
        let range = Range { start, end };
        let statement = Statement::print(expression, range);
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

        let start = expression.range.start;
        let end = token.range.end;
        let range = Range { start, end };
        let statement = Statement::expression(expression, range);
        Some(Ok(statement))
    }

    fn expression(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Result<Expression<'a>, ParseError>> {
        let lhs = match self.equality()? {
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
                match unescape(raw) {
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

fn unescape(raw: &str) -> Result<Cow<'_, str>, Range<usize>> {
    let Some(mut cursor) = raw.find('\\') else {
        let string = Cow::Borrowed(raw);
        return Ok(string);
    };

    let raw_len = raw.len();
    let mut string = String::with_capacity(raw_len);

    let prefix = &raw[..cursor];
    string.push_str(prefix);

    loop {
        let remaining = &raw[cursor..];
        let remaining_bytes = remaining.as_bytes();
        let Some(byte_1) = remaining_bytes.get(1) else {
            // This branch is unreachable when called from `Parser::primary`. It exists only
            // for correctness should this method ever be called directly, even though it is
            // not intended to.
            let start = cursor;
            let end = raw_len;
            let range = Range { start, end };
            return Err(range);
        };
        let (unescaped, sequence_len) = match byte_1 {
            b'0' => ('\0', 2),
            b't' => ('\t', 2),
            b'n' => ('\n', 2),
            b'r' => ('\r', 2),
            b'"' => ('"', 2),
            b'\\' => ('\\', 2),
            b'x' => {
                let Some(bytes) = remaining_bytes.get(2..4) else {
                    let start = cursor;
                    let end = raw_len;
                    let range = Range { start, end };
                    return Err(range);
                };
                let mut code = 0;
                for (offset, byte) in (2..).zip(bytes) {
                    match byte {
                        b'0'..=b'9' => {
                            code = (code << 4) | (byte - b'0');
                        }
                        b'a'..=b'f' => {
                            code = (code << 4) | (byte - b'a' + 10);
                        }
                        b'A'..=b'F' => {
                            code = (code << 4) | (byte - b'A' + 10);
                        }
                        _ => {
                            let char_len = byte.leading_ones().max(1) as usize;
                            let start = cursor;
                            let end = cursor + offset + char_len;
                            let range = Range { start, end };
                            return Err(range);
                        }
                    }
                }
                if code > 0x7F {
                    let start = cursor;
                    let end = cursor + 4;
                    let range = Range { start, end };
                    return Err(range);
                }
                (code as char, 4)
            }
            b'u' => {
                let [byte_2, byte_3, ..] = &remaining_bytes[2..] else {
                    let start = cursor;
                    let end = raw_len;
                    let range = Range { start, end };
                    return Err(range);
                };
                if *byte_2 != b'{' {
                    let char_len = byte_2.leading_ones().max(1) as usize;
                    let start = cursor;
                    let end = cursor + 2 + char_len;
                    let range = Range { start, end };
                    return Err(range);
                }
                let mut code = match byte_3 {
                    b'0'..=b'9' => (byte_3 - b'0') as u32,
                    b'a'..=b'f' => (byte_3 - b'a' + 10) as u32,
                    b'A'..=b'F' => (byte_3 - b'A' + 10) as u32,
                    _ => {
                        let char_len = byte_3.leading_ones().max(1) as usize;
                        let start = cursor;
                        let end = cursor + 3 + char_len;
                        let range = Range { start, end };
                        return Err(range);
                    }
                };
                let mut offset = 4;
                loop {
                    let Some(byte) = remaining_bytes.get(offset) else {
                        let start = cursor;
                        let end = raw_len;
                        let range = Range { start, end };
                        return Err(range);
                    };
                    if offset == 10 {
                        let char_len = byte.leading_ones().max(1) as usize;
                        let start = cursor;
                        let end = cursor + offset + char_len;
                        let range = Range { start, end };
                        return Err(range);
                    }
                    match byte {
                        b'0'..=b'9' => {
                            code = (code << 4) | (byte - b'0') as u32;
                        }
                        b'a'..=b'f' => {
                            code = (code << 4) | (byte - b'a' + 10) as u32;
                        }
                        b'A'..=b'F' => {
                            code = (code << 4) | (byte - b'A' + 10) as u32;
                        }
                        b'}' => {
                            let sequence_len = offset + 1;
                            let Some(char) = char::from_u32(code) else {
                                let start = cursor;
                                let end = cursor + sequence_len;
                                let range = Range { start, end };
                                return Err(range);
                            };
                            break (char, sequence_len);
                        }
                        _ => {
                            let char_len = byte.leading_ones().max(1) as usize;
                            let start = cursor;
                            let end = cursor + offset + char_len;
                            let range = Range { start, end };
                            return Err(range);
                        }
                    }
                    offset += 1;
                }
            }
            _ => {
                let char_len = byte_1.leading_ones().max(1) as usize;
                let start = cursor;
                let end = cursor + 1 + char_len;
                let range = Range { start, end };
                return Err(range);
            }
        };

        let remaining = &remaining[sequence_len..];
        match remaining.find('\\') {
            None => {
                let plain_text = remaining;
                string.push(unescaped);
                string.push_str(plain_text);
                string.shrink_to_fit();
                let string = Cow::Owned(string);
                return Ok(string);
            }
            Some(offset) => {
                let plain_text = &remaining[..offset];
                string.push(unescaped);
                string.push_str(plain_text);
                cursor += sequence_len + offset;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape() {
        let raw = "\tHello,\n\tworld!\n";
        let Ok(Cow::Borrowed(unescaped)) = unescape(raw) else {
            unreachable!()
        };
        let expected = raw;
        assert_eq!(unescaped, expected);

        let raw = r"\tHe\x6c\x6Co,\n\twor\u{006c}d!\n";
        let Ok(Cow::Owned(unescaped)) = unescape(raw) else {
            unreachable!()
        };
        let expected = "\tHello,\n\tworld!\n";
        assert_eq!(unescaped, expected);

        let raw = r"\\";
        let Ok(Cow::Owned(unescaped)) = unescape(raw) else {
            unreachable!()
        };
        let expected = "\\";
        assert_eq!(unescaped, expected);

        let raw = r"\";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 1 };
        assert_eq!(range, expected);

        let raw = r"\x";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 2 };
        assert_eq!(range, expected);

        let raw = r"\x0";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 3 };
        assert_eq!(range, expected);

        let raw = r"\x0文";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 6 };
        assert_eq!(range, expected);

        let raw = r"\xFF";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 4 };
        assert_eq!(range, expected);

        let raw = r"\x000";
        let Ok(Cow::Owned(unescaped)) = unescape(raw) else {
            unreachable!()
        };
        let expected = "\x000";
        assert_eq!(unescaped, expected);

        let raw = r"\u";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 2 };
        assert_eq!(range, expected);

        let raw = r"\u文";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 5 };
        assert_eq!(range, expected);

        let raw = r"\u{";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 3 };
        assert_eq!(range, expected);

        let raw = r"\u{0";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 4 };
        assert_eq!(range, expected);

        let raw = r"\u{0文";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 7 };
        assert_eq!(range, expected);

        let raw = r"\u{6587}";
        let Ok(Cow::Owned(unescaped)) = unescape(raw) else {
            unreachable!()
        };
        let expected = "文";
        assert_eq!(unescaped, expected);

        let raw = r"\u{006587}";
        let Ok(Cow::Owned(unescaped)) = unescape(raw) else {
            unreachable!()
        };
        let expected = "文";
        assert_eq!(unescaped, expected);

        let raw = r"\u{0006587}";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 11 };
        assert_eq!(range, expected);

        let raw = r"\u{110000}";
        let Err(range) = unescape(raw) else {
            unreachable!()
        };
        let expected = Range { start: 0, end: 10 };
        assert_eq!(range, expected);
    }
}
