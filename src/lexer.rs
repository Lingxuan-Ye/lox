use crate::token::{Keyword, Token, TokenKind};
use core::error::Error;
use core::fmt;
use core::range::Range;

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

    pub fn source(&self) -> &'a str {
        self.source
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

impl Iterator for Lexer<'_> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.cursor;
            let byte = self.next_byte()?;
            match byte {
                b'(' => {
                    let kind = TokenKind::LParen;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b')' => {
                    let kind = TokenKind::RParen;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'{' => {
                    let kind = TokenKind::LBrace;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'}' => {
                    let kind = TokenKind::RBrace;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b';' => {
                    let kind = TokenKind::Semicolon;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b',' => {
                    let kind = TokenKind::Comma;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'.' => {
                    let kind = TokenKind::Dot;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'+' => {
                    let kind = TokenKind::Plus;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'-' => {
                    let kind = TokenKind::Minus;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'*' => {
                    let kind = TokenKind::Star;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
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
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    }
                }
                b'!' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::BangEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    } else {
                        let kind = TokenKind::Bang;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    }
                }
                b'=' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::EqualEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    } else {
                        let kind = TokenKind::Equal;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    }
                }
                b'<' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::LessEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    } else {
                        let kind = TokenKind::Less;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    }
                }
                b'>' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        let kind = TokenKind::GreaterEqual;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
                    } else {
                        let kind = TokenKind::Greater;
                        let end = self.cursor;
                        let range = Range { start, end };
                        let token = Token { kind, range };
                        return Some(Ok(token));
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
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                0x80..=0xFF => {
                    let mut chars = self.source[start..].chars();
                    let char = chars.next().unwrap_or_else(|| unreachable!());
                    self.cursor = start + char.len_utf8();
                    if !unicode_ident::is_xid_start(char) {
                        let end = self.cursor;
                        let range = Range { start, end };
                        let error = LexError::UnexpectedCharacter { char, range };
                        return Some(Err(error));
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
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                b'"' => loop {
                    match self.next_byte() {
                        None => {
                            let error = LexError::UnterminatedString;
                            return Some(Err(error));
                        }
                        Some(b'"') => {
                            let end = self.cursor;
                            let range = Range { start, end };
                            let kind = TokenKind::String;
                            let token = Token { kind, range };
                            return Some(Ok(token));
                        }
                        Some(b'\\') => {
                            if self.next_byte().is_none() {
                                let error = LexError::UnterminatedString;
                                return Some(Err(error));
                            }
                        }
                        Some(_) => (),
                    }
                },
                _ if byte.is_ascii_digit() => {
                    for byte in &self.source.as_bytes()[self.cursor..] {
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
                        for byte in &self.source.as_bytes()[self.cursor..] {
                            if !byte.is_ascii_digit() {
                                break;
                            }
                            self.cursor += 1;
                        }
                    }
                    let kind = TokenKind::Number;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let token = Token { kind, range };
                    return Some(Ok(token));
                }
                0x00..=0x7F => {
                    let char = byte as char;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let error = LexError::UnexpectedCharacter { char, range };
                    return Some(Err(error));
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { char: char, range: Range<usize> },
    UnterminatedString,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { char, range } => {
                let Range { start, end } = range;
                write!(f, "unexpected character '{char}' at {start}..{end}")
            }
            Self::UnterminatedString => {
                write!(f, "unterminated string")
            }
        }
    }
}

impl Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = r#"
            fun ask(question) {
                return 42;
            }

            fun main() {
                var question = "
                    The Answer to the Great Question of Life,
                    the Universe and Everything.
                ";
                var answer = ask(question);
                print answer;
            }
        "#;
        let lexer = Lexer::new(source);
        let expected = [
            TokenKind::Keyword(Keyword::Fun),
            TokenKind::Identifier,
            TokenKind::LParen,
            TokenKind::Identifier,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Keyword(Keyword::Return),
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Keyword(Keyword::Fun),
            TokenKind::Identifier,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Keyword(Keyword::Var),
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::String,
            TokenKind::Semicolon,
            TokenKind::Keyword(Keyword::Var),
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::Identifier,
            TokenKind::LParen,
            TokenKind::Identifier,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Keyword(Keyword::Print),
            TokenKind::Identifier,
            TokenKind::Semicolon,
            TokenKind::RBrace,
        ];
        for (token, expected) in lexer.map(Result::unwrap).zip(expected) {
            assert_eq!(token.kind, expected);
        }
    }
}
