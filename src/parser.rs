use crate::expression::{BinaryOperator, Expression, ExpressionKind, Literal, UnaryOperator};
use crate::lexer::{Lexer, LexerError};
use crate::token::{Keyword, Token, TokenKind};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use core::error::Error;
use core::fmt;
use core::range::Range;

// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( "<" | "<=" | ">" | ">=" ) term )* ;
// term           → factor ( ( "+" | "-" ) factor )* ;
// factor         → unary ( ( "*" | "/" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → "(" expression ")" | STRING | NUMBER | "true" | "false"
//                | "nil" ;

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Option<Result<Token, LexerError>>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let peeked = None;
        Self { lexer, peeked }
    }

    pub fn source(&self) -> &'a str {
        self.lexer.source()
    }

    pub fn parse(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        self.expression()
    }

    pub fn syncronize(&mut self) {
        while let Some(token) = self.peek_token() {
            if let Ok(token) = token {
                match token.kind {
                    TokenKind::Semicolon => {
                        self.next_token();
                        break;
                    }
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
                    _ => {
                        self.next_token();
                    }
                }
            }
        }
    }

    fn next_token(&mut self) -> Option<Result<Token, LexerError>> {
        match self.peeked.take() {
            None => self.lexer.next(),
            Some(peeked) => peeked,
        }
    }

    fn peek_token(&mut self) -> Option<&Result<Token, LexerError>> {
        self.peeked
            .get_or_insert_with(|| self.lexer.next())
            .as_ref()
    }
}

impl<'a> Parser<'a> {
    fn expression(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        let mut expression = match self.comparison()? {
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
                    let error = ParseError::from(error);
                    return Some(Err(error));
                }
                Some(Ok(token)) => token,
            };

            let operator = match token.kind {
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                TokenKind::EqualEqual => BinaryOperator::Equal,
                _ => break,
            };

            self.next_token();

            match self.comparison() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => {
                    let start = expression.range.start;
                    let end = rhs.range.end;
                    let range = Range { start, end };
                    let lhs = Box::new(expression);
                    let rhs = Box::new(rhs);
                    let kind = ExpressionKind::Binary { operator, lhs, rhs };
                    expression = Expression { kind, range };
                }
            }
        }

        Some(Ok(expression))
    }

    fn comparison(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        let mut expression = match self.term()? {
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
                    let error = ParseError::from(error);
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

            match self.term() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => {
                    let start = expression.range.start;
                    let end = rhs.range.end;
                    let range = Range { start, end };
                    let lhs = Box::new(expression);
                    let rhs = Box::new(rhs);
                    let kind = ExpressionKind::Binary { operator, lhs, rhs };
                    expression = Expression { kind, range };
                }
            }
        }

        Some(Ok(expression))
    }

    fn term(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        let mut expression = match self.factor()? {
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
                    let error = ParseError::from(error);
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

            match self.factor() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => {
                    let start = expression.range.start;
                    let end = rhs.range.end;
                    let range = Range { start, end };
                    let lhs = Box::new(expression);
                    let rhs = Box::new(rhs);
                    let kind = ExpressionKind::Binary { operator, lhs, rhs };
                    expression = Expression { kind, range };
                }
            }
        }

        Some(Ok(expression))
    }

    fn factor(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        let mut expression = match self.unary()? {
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
                    let error = ParseError::from(error);
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

            match self.unary() {
                None => {
                    let error = ParseError::UnexpectedEndOfInput;
                    return Some(Err(error));
                }
                Some(Err(error)) => return Some(Err(error)),
                Some(Ok(rhs)) => {
                    let start = expression.range.start;
                    let end = rhs.range.end;
                    let range = Range { start, end };
                    let lhs = Box::new(expression);
                    let rhs = Box::new(rhs);
                    let kind = ExpressionKind::Binary { operator, lhs, rhs };
                    expression = Expression { kind, range };
                }
            }
        }

        Some(Ok(expression))
    }

    fn unary(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        let Ok(token) = self.peek_token()? else {
            let Some(Err(error)) = self.next_token() else {
                unreachable!()
            };
            let error = ParseError::from(error);
            return Some(Err(error));
        };

        let operator = match token.kind {
            TokenKind::Minus => UnaryOperator::Neg,
            TokenKind::Bang => UnaryOperator::Not,
            _ => return self.primary(),
        };

        let start = token.range.start;

        self.next_token();

        match self.unary() {
            None => {
                let error = ParseError::UnexpectedEndOfInput;
                Some(Err(error))
            }
            Some(Err(error)) => Some(Err(error)),
            Some(Ok(rhs)) => {
                let end = rhs.range.end;
                let range = Range { start, end };
                let rhs = Box::new(rhs);
                let kind = ExpressionKind::Unary { operator, rhs };
                let expression = Expression { kind, range };
                Some(Ok(expression))
            }
        }
    }

    fn primary(&mut self) -> Option<Result<Expression<'a>, ParseError<'a>>> {
        let token = match self.next_token()? {
            Err(error) => {
                let error = ParseError::from(error);
                return Some(Err(error));
            }
            Ok(token) => token,
        };

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
                        let error = ParseError::from(error);
                        return Some(Err(error));
                    }
                    Some(Ok(token)) => token,
                };

                match token.kind {
                    TokenKind::RParen => {
                        let expression = Box::new(expression);
                        let kind = ExpressionKind::Grouping(expression);
                        let end = token.range.end;
                        let range = Range { start, end };
                        let expression = Expression { kind, range };
                        Some(Ok(expression))
                    }
                    _ => {
                        let error = ParseError::UnexpectedToken(token);
                        Some(Err(error))
                    }
                }
            }

            TokenKind::String => {
                let start = token.range.start + 1;
                let end = token.range.end - 1;
                let raw_range = Range { start, end };
                let raw = &self.source()[raw_range];

                let mut string = Cow::Borrowed("");
                let mut plain_text_range = Range::default();

                loop {
                    if plain_text_range.end == raw.len() {
                        let plain_text = &raw[plain_text_range];
                        match &mut string {
                            Cow::Borrowed(_) => {
                                string = Cow::Borrowed(plain_text);
                            }
                            Cow::Owned(inner) => {
                                inner.push_str(plain_text);
                                inner.shrink_to_fit();
                            }
                        }
                        break;
                    }

                    let remaining = &raw[plain_text_range.end..];
                    let remaining_bytes = remaining.as_bytes();

                    let byte_0 = remaining_bytes[0];
                    if byte_0 != b'\\' {
                        let char_len = byte_0.leading_ones().max(1) as usize;
                        plain_text_range.end += char_len;
                        continue;
                    }

                    let mut commit = |unescaped, len| {
                        let plain_text = &raw[plain_text_range];
                        match &mut string {
                            Cow::Borrowed(_) => {
                                let inner = format!("{plain_text}{unescaped}");
                                string = Cow::Owned(inner);
                            }
                            Cow::Owned(inner) => {
                                inner.push_str(plain_text);
                                inner.push(unescaped);
                            }
                        }
                        plain_text_range.start = plain_text_range.end + len;
                        plain_text_range.end = plain_text_range.start;
                    };

                    let byte_1 = remaining_bytes[1];
                    match byte_1 {
                        b'0' => commit('\0', 2),
                        b't' => commit('\t', 2),
                        b'n' => commit('\n', 2),
                        b'r' => commit('\r', 2),
                        b'"' => commit('"', 2),
                        b'\\' => commit('\\', 2),
                        b'x' => {
                            let Some(bytes) = remaining_bytes.get(2..4) else {
                                let sequence = remaining;
                                let start = raw_range.start + plain_text_range.end;
                                let end = raw_range.end;
                                let range = Range { start, end };
                                let error = ParseError::InvalidEscapeSequence { sequence, range };
                                return Some(Err(error));
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
                                        let len = offset + char_len;
                                        let sequence = &remaining[..len];
                                        let start = raw_range.start + plain_text_range.end;
                                        let end = start + len;
                                        let range = Range { start, end };
                                        let error =
                                            ParseError::InvalidEscapeSequence { sequence, range };
                                        return Some(Err(error));
                                    }
                                }
                            }
                            if code > 0x7F {
                                let sequence = &remaining[..4];
                                let start = raw_range.start + plain_text_range.end;
                                let end = start + 4;
                                let range = Range { start, end };
                                let error = ParseError::InvalidEscapeSequence { sequence, range };
                                return Some(Err(error));
                            }
                            commit(code as char, 4);
                        }
                        b'u' => {
                            let [byte_2, byte_3, ..] = &remaining_bytes[2..] else {
                                let sequence = remaining;
                                let start = raw_range.start + plain_text_range.end;
                                let end = raw_range.end;
                                let range = Range { start, end };
                                let error = ParseError::InvalidEscapeSequence { sequence, range };
                                return Some(Err(error));
                            };
                            if *byte_2 != b'{' {
                                let char_len = byte_2.leading_ones().max(1) as usize;
                                let len = 2 + char_len;
                                let sequence = &remaining[..len];
                                let start = raw_range.start + plain_text_range.end;
                                let end = start + len;
                                let range = Range { start, end };
                                let error = ParseError::InvalidEscapeSequence { sequence, range };
                                return Some(Err(error));
                            }
                            let mut code = match byte_3 {
                                b'0'..=b'9' => (byte_3 - b'0') as u32,
                                b'a'..=b'f' => (byte_3 - b'a' + 10) as u32,
                                b'A'..=b'F' => (byte_3 - b'A' + 10) as u32,
                                _ => {
                                    let char_len = byte_3.leading_ones().max(1) as usize;
                                    let len = 3 + char_len;
                                    let sequence = &remaining[..len];
                                    let start = raw_range.start + plain_text_range.end;
                                    let end = start + len;
                                    let range = Range { start, end };
                                    let error =
                                        ParseError::InvalidEscapeSequence { sequence, range };
                                    return Some(Err(error));
                                }
                            };
                            let mut offset = 4;
                            loop {
                                let byte = match remaining_bytes.get(offset) {
                                    Some(byte) if offset < 10 => byte,
                                    _ => {
                                        let sequence = &remaining[..offset];
                                        let start = raw_range.start + plain_text_range.end;
                                        let end = start + offset;
                                        let range = Range { start, end };
                                        let error =
                                            ParseError::InvalidEscapeSequence { sequence, range };
                                        return Some(Err(error));
                                    }
                                };
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
                                        let len = offset + 1;
                                        let Some(char) = char::from_u32(code) else {
                                            let sequence = &remaining[..len];
                                            let start = raw_range.start + plain_text_range.end;
                                            let end = start + len;
                                            let range = Range { start, end };
                                            let error = ParseError::InvalidEscapeSequence {
                                                sequence,
                                                range,
                                            };
                                            return Some(Err(error));
                                        };
                                        commit(char, len);
                                        break;
                                    }
                                    _ => {
                                        let char_len = byte.leading_ones().max(1) as usize;
                                        let len = offset + char_len;
                                        let sequence = &remaining[..len];
                                        let start = raw_range.start + plain_text_range.end;
                                        let end = start + len;
                                        let range = Range { start, end };
                                        let error =
                                            ParseError::InvalidEscapeSequence { sequence, range };
                                        return Some(Err(error));
                                    }
                                }
                                offset += 1;
                            }
                        }
                        _ => {
                            let char_len = byte_1.leading_ones().max(1) as usize;
                            let len = 1 + char_len;
                            let sequence = &remaining[..len];
                            let start = raw_range.start + plain_text_range.end;
                            let end = start + len;
                            let range = Range { start, end };
                            let error = ParseError::InvalidEscapeSequence { sequence, range };
                            return Some(Err(error));
                        }
                    }
                }

                let literal = Literal::String(string);
                let kind = ExpressionKind::Literal(literal);
                let range = token.range;
                let expression = Expression { kind, range };
                Some(Ok(expression))
            }

            TokenKind::Number => {
                let number = self.source()[token.range]
                    .parse()
                    .unwrap_or_else(|_| unreachable!());
                let literal = Literal::Number(number);
                let kind = ExpressionKind::Literal(literal);
                let range = token.range;
                let expression = Expression { kind, range };
                Some(Ok(expression))
            }

            TokenKind::Keyword(Keyword::True) => {
                let literal = Literal::Boolean(true);
                let kind = ExpressionKind::Literal(literal);
                let range = token.range;
                let expression = Expression { kind, range };
                Some(Ok(expression))
            }

            TokenKind::Keyword(Keyword::False) => {
                let literal = Literal::Boolean(false);
                let kind = ExpressionKind::Literal(literal);
                let range = token.range;
                let expression = Expression { kind, range };
                Some(Ok(expression))
            }

            TokenKind::Keyword(Keyword::Nil) => {
                let literal = Literal::Nil;
                let kind = ExpressionKind::Literal(literal);
                let range = token.range;
                let expression = Expression { kind, range };
                Some(Ok(expression))
            }

            _ => {
                let error = ParseError::UnexpectedToken(token);
                Some(Err(error))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    LexerError(LexerError),
    UnexpectedEndOfInput,
    UnexpectedToken(Token),
    InvalidEscapeSequence {
        sequence: &'a str,
        range: Range<usize>,
    },
}

impl From<LexerError> for ParseError<'_> {
    fn from(value: LexerError) -> Self {
        Self::LexerError(value)
    }
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerError(error) => write!(f, "{error}"),
            Self::UnexpectedEndOfInput => write!(f, "unexpected end of input"),
            Self::UnexpectedToken(token) => write!(f, "unexpected token {token}"),
            Self::InvalidEscapeSequence { sequence, range } => {
                let Range { start, end } = range;
                write!(f, "invalid escape sequence '{sequence}' at {start}..{end}")
            }
        }
    }
}

impl Error for ParseError<'_> {}
