pub use crate::error::LexerError;
pub use crate::token::{Keyword, Token, TokenKind};

use std::borrow::Cow;
use std::range::Range;

mod error;
mod token;

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let cursor = 0;
        Self { source, cursor }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.source.as_bytes().get(self.cursor).copied()?;
        self.cursor += 1;
        Some(byte)
    }

    fn peek_byte<const N: usize>(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor + N).copied()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexerError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.cursor;
            let byte = self.next_byte()?;
            match byte {
                b'(' => {
                    let kind = TokenKind::LParen;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b')' => {
                    let kind = TokenKind::RParen;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'{' => {
                    let kind = TokenKind::LBrace;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'}' => {
                    let kind = TokenKind::RBrace;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b',' => {
                    let kind = TokenKind::Comma;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'.' => {
                    let kind = TokenKind::Dot;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'-' => {
                    let kind = TokenKind::Minus;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'+' => {
                    let kind = TokenKind::Plus;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b';' => {
                    let kind = TokenKind::Semicolon;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'*' => {
                    let kind = TokenKind::Star;
                    let end = self.cursor;
                    let range = Range { start, end };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'!' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::BangEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    } else {
                        let kind = TokenKind::Bang;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    }
                }
                b'=' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::EqualEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    } else {
                        let kind = TokenKind::Equal;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    }
                }
                b'<' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::LessEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    } else {
                        let kind = TokenKind::Less;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    }
                }
                b'>' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::GreaterEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    } else {
                        let kind = TokenKind::Greater;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    }
                }
                b'/' => {
                    if self.peek_byte::<0>() == Some(b'/') {
                        self.cursor += 1;
                        while self.next_byte()? != b'\n' {}
                        continue;
                    } else {
                        let kind = TokenKind::Slash;
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Ok(Token::new(kind, range)));
                    }
                }
                _ if byte.is_ascii_whitespace() => continue,
                _ if byte.is_ascii_alphabetic() || byte == b'_' => {
                    for char in self.source[self.cursor..].chars() {
                        if !unicode_ident::is_xid_continue(char) {
                            break;
                        }
                        self.cursor += char.len_utf8();
                    }
                    let end = self.cursor;
                    let range = Range { start, end };
                    let lexeme = &self.source[range];
                    let kind = match Keyword::from_lexeme(lexeme) {
                        Some(keyword) => TokenKind::Keyword(keyword),
                        None => TokenKind::Identifier,
                    };
                    return Some(Ok(Token::new(kind, range)));
                }
                0x80..=0xFF => {
                    let mut chars = self.source[start..].chars();
                    let char = chars.next().unwrap_or_else(|| unreachable!());
                    self.cursor = start + char.len_utf8();
                    if !unicode_ident::is_xid_start(char) {
                        let end = self.cursor;
                        let range = Range { start, end };
                        return Some(Err(LexerError::UnexpectedCharacter { range, char }));
                    }
                    for char in chars {
                        if !unicode_ident::is_xid_continue(char) {
                            break;
                        }
                        self.cursor += char.len_utf8();
                    }
                    let end = self.cursor;
                    let range = Range { start, end };
                    let lexeme = &self.source[range];
                    let kind = match Keyword::from_lexeme(lexeme) {
                        Some(keyword) => TokenKind::Keyword(keyword),
                        None => TokenKind::Identifier,
                    };
                    return Some(Ok(Token::new(kind, range)));
                }
                b'"' => {
                    let mut string = Cow::Borrowed("");
                    let mut plain_text_range = Range {
                        start: self.cursor,
                        end: self.cursor,
                    };
                    loop {
                        match self.next_byte() {
                            None => {
                                return Some(Err(LexerError::UnterminatedString));
                            }
                            Some(b'"') => {
                                let plain_text = &self.source[plain_text_range];
                                match &mut string {
                                    Cow::Borrowed(_) => string = Cow::Borrowed(plain_text),
                                    Cow::Owned(inner) => {
                                        inner.push_str(plain_text);
                                        inner.shrink_to_fit();
                                    }
                                }
                                let kind = TokenKind::String(string);
                                let end = self.cursor;
                                let range = Range { start, end };
                                return Some(Ok(Token::new(kind, range)));
                            }
                            Some(b'\\') => match self.next_byte() {
                                None => {
                                    return Some(Err(LexerError::UnterminatedString));
                                }
                                Some(b'0') => {
                                    let plain_text = &self.source[plain_text_range];
                                    match &mut string {
                                        Cow::Borrowed(_) => {
                                            let inner = format!("{plain_text}\0");
                                            string = Cow::Owned(inner);
                                        }
                                        Cow::Owned(inner) => {
                                            inner.push_str(plain_text);
                                            inner.push('\0');
                                        }
                                    }
                                    plain_text_range.start = self.cursor;
                                    plain_text_range.end = self.cursor;
                                }
                                Some(b't') => {
                                    let plain_text = &self.source[plain_text_range];
                                    match &mut string {
                                        Cow::Borrowed(_) => {
                                            let inner = format!("{plain_text}\t");
                                            string = Cow::Owned(inner);
                                        }
                                        Cow::Owned(inner) => {
                                            inner.push_str(plain_text);
                                            inner.push('\t');
                                        }
                                    }
                                    plain_text_range.start = self.cursor;
                                    plain_text_range.end = self.cursor;
                                }
                                Some(b'n') => {
                                    let plain_text = &self.source[plain_text_range];
                                    match &mut string {
                                        Cow::Borrowed(_) => {
                                            let inner = format!("{plain_text}\n");
                                            string = Cow::Owned(inner);
                                        }
                                        Cow::Owned(inner) => {
                                            inner.push_str(plain_text);
                                            inner.push('\n');
                                        }
                                    }
                                    plain_text_range.start = self.cursor;
                                    plain_text_range.end = self.cursor;
                                }
                                Some(b'r') => {
                                    let plain_text = &self.source[plain_text_range];
                                    match &mut string {
                                        Cow::Borrowed(_) => {
                                            let inner = format!("{plain_text}\r");
                                            string = Cow::Owned(inner);
                                        }
                                        Cow::Owned(inner) => {
                                            inner.push_str(plain_text);
                                            inner.push('\r');
                                        }
                                    }
                                    plain_text_range.start = self.cursor;
                                    plain_text_range.end = self.cursor;
                                }
                                Some(b'x') => {
                                    let Some(bytes) =
                                        self.source.as_bytes().get(self.cursor..self.cursor + 2)
                                    else {
                                        self.cursor = self.source.len();
                                        return Some(Err(LexerError::UnterminatedString));
                                    };
                                    let mut result = 0;
                                    for byte in bytes {
                                        match byte {
                                            b'0'..=b'9' => {
                                                result = (result << 4) | (byte - b'0');
                                            }
                                            b'a'..=b'f' => {
                                                result = (result << 4) | (byte - b'a' + 10);
                                            }
                                            b'A'..=b'F' => {
                                                result = (result << 4) | (byte - b'A' + 10);
                                            }
                                            _ => {
                                                let escseq_start = plain_text_range.end;
                                                let char = self.source[self.cursor..]
                                                    .chars()
                                                    .next()
                                                    .unwrap_or_else(|| unreachable!());
                                                self.cursor += char.len_utf8();
                                                let range = Range {
                                                    start: escseq_start,
                                                    end: self.cursor,
                                                };
                                                let sequence = &self.source[range];
                                                return Some(Err(
                                                    LexerError::InvalidEscapeSequence {
                                                        sequence,
                                                        range,
                                                    },
                                                ));
                                            }
                                        }
                                        self.cursor += 1;
                                    }
                                    if result > 0x7F {
                                        let escseq_start = plain_text_range.end;
                                        let range = Range {
                                            start: escseq_start,
                                            end: self.cursor,
                                        };
                                        let sequence = &self.source[range];
                                        return Some(Err(LexerError::InvalidEscapeSequence {
                                            sequence,
                                            range,
                                        }));
                                    }
                                    let plain_text = &self.source[plain_text_range];
                                    let char = result as char;
                                    match &mut string {
                                        Cow::Borrowed(_) => {
                                            let inner = format!("{plain_text}{char}");
                                            string = Cow::Owned(inner);
                                        }
                                        Cow::Owned(inner) => {
                                            inner.push_str(plain_text);
                                            inner.push(char);
                                        }
                                    }
                                    plain_text_range.start = self.cursor;
                                    plain_text_range.end = self.cursor;
                                }
                                Some(b'u') => {
                                    match self.next_byte() {
                                        None => {
                                            return Some(Err(LexerError::UnterminatedString));
                                        }
                                        Some(b'{') => (),
                                        Some(_) => {
                                            let escseq_start = plain_text_range.end;
                                            let char_start = escseq_start + 2;
                                            let char = self.source[char_start..]
                                                .chars()
                                                .next()
                                                .unwrap_or_else(|| unreachable!());
                                            self.cursor = char_start + char.len_utf8();
                                            let range = Range {
                                                start: escseq_start,
                                                end: self.cursor,
                                            };
                                            let sequence = &self.source[range];
                                            return Some(Err(LexerError::InvalidEscapeSequence {
                                                sequence,
                                                range,
                                            }));
                                        }
                                    }
                                    let mut result = match self.next_byte() {
                                        None => {
                                            return Some(Err(LexerError::UnterminatedString));
                                        }
                                        Some(b'0'..=b'9') => (byte - b'0') as u32,
                                        Some(b'a'..=b'f') => (byte - b'a' + 10) as u32,
                                        Some(b'A'..=b'F') => (byte - b'A' + 10) as u32,
                                        Some(_) => {
                                            let escseq_start = plain_text_range.end;
                                            let char_start = escseq_start + 3;
                                            let char = self.source[char_start..]
                                                .chars()
                                                .next()
                                                .unwrap_or_else(|| unreachable!());
                                            self.cursor = char_start + char.len_utf8();
                                            let range = Range {
                                                start: escseq_start,
                                                end: self.cursor,
                                            };
                                            let sequence = &self.source[range];
                                            return Some(Err(LexerError::InvalidEscapeSequence {
                                                sequence,
                                                range,
                                            }));
                                        }
                                    };
                                    for byte in self.source.as_bytes()[self.cursor..].iter().take(5)
                                    {
                                        match byte {
                                            b'0'..=b'9' => {
                                                result = (result << 4) | (byte - b'0') as u32;
                                            }
                                            b'a'..=b'f' => {
                                                result = (result << 4) | (byte - b'a' + 10) as u32;
                                            }
                                            b'A'..=b'F' => {
                                                result = (result << 4) | (byte - b'A' + 10) as u32;
                                            }
                                            _ => {
                                                break;
                                            }
                                        }
                                        self.cursor += 1;
                                    }
                                    if result > 0x10FFFF {
                                        let escseq_start = plain_text_range.end;
                                        let range = Range {
                                            start: escseq_start,
                                            end: self.cursor,
                                        };
                                        let sequence = &self.source[range];
                                        return Some(Err(LexerError::InvalidEscapeSequence {
                                            sequence,
                                            range,
                                        }));
                                    }
                                    match self.next_byte() {
                                        None => {
                                            return Some(Err(LexerError::UnterminatedString));
                                        }
                                        Some(b'}') => (),
                                        Some(_) => {
                                            let escseq_start = plain_text_range.end;
                                            let char_start = self.cursor - 1;
                                            let char = self.source[char_start..]
                                                .chars()
                                                .next()
                                                .unwrap_or_else(|| unreachable!());
                                            self.cursor = char_start + char.len_utf8();
                                            let range = Range {
                                                start: escseq_start,
                                                end: self.cursor,
                                            };
                                            let sequence = &self.source[range];
                                            return Some(Err(LexerError::InvalidEscapeSequence {
                                                sequence,
                                                range,
                                            }));
                                        }
                                    }
                                    let plain_text = &self.source[plain_text_range];
                                    let char = match char::from_u32(result) {
                                        Some(char) => char,
                                        None => {
                                            let escseq_start = plain_text_range.end;
                                            let range = Range {
                                                start: escseq_start,
                                                end: self.cursor,
                                            };
                                            let sequence = &self.source[range];
                                            return Some(Err(LexerError::InvalidEscapeSequence {
                                                sequence,
                                                range,
                                            }));
                                        }
                                    };
                                    match &mut string {
                                        Cow::Borrowed(_) => {
                                            let inner = format!("{plain_text}{char}");
                                            string = Cow::Owned(inner);
                                        }
                                        Cow::Owned(inner) => {
                                            inner.push_str(plain_text);
                                            inner.push(char);
                                        }
                                    }
                                    plain_text_range.start = self.cursor;
                                    plain_text_range.end = self.cursor;
                                }
                                Some(_) => {
                                    let escseq_start = plain_text_range.end;
                                    let char_start = escseq_start + 1;
                                    let char = self.source[char_start..]
                                        .chars()
                                        .next()
                                        .unwrap_or_else(|| unreachable!());
                                    self.cursor = char_start + char.len_utf8();
                                    let range = Range {
                                        start: escseq_start,
                                        end: self.cursor,
                                    };
                                    let sequence = &self.source[range];
                                    return Some(Err(LexerError::InvalidEscapeSequence {
                                        sequence,
                                        range,
                                    }));
                                }
                            },
                            Some(_) => {
                                plain_text_range.end = self.cursor;
                                continue;
                            }
                        }
                    }
                }
                _ if byte.is_ascii_digit() => {
                    let bytes = self.source.as_bytes();
                    for byte in &bytes[self.cursor..] {
                        if !byte.is_ascii_digit() {
                            break;
                        }
                        self.cursor += 1;
                    }
                    if self.peek_byte::<0>() == Some(b'.')
                        && self
                            .peek_byte::<1>()
                            .is_some_and(|byte| byte.is_ascii_digit())
                    {
                        self.cursor += 2;
                        for byte in &bytes[self.cursor..] {
                            if !byte.is_ascii_digit() {
                                break;
                            }
                            self.cursor += 1;
                        }
                    }
                    let end = self.cursor;
                    let range = Range { start, end };
                    let number = self.source[range]
                        .parse::<f64>()
                        .unwrap_or_else(|_| unreachable!());
                    let kind = TokenKind::Number(number);
                    return Some(Ok(Token::new(kind, range)));
                }
                0x00..=0x7F => {
                    let end = self.cursor;
                    let range = Range { start, end };
                    let char = byte as char;
                    return Some(Err(LexerError::UnexpectedCharacter { range, char }));
                }
            }
        }
    }
}
