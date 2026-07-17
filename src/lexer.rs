use crate::token::{Keyword, Token, TokenKind};
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

    fn peek_byte<const N: usize>(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor + N).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.source.as_bytes().get(self.cursor).copied()?;
        self.cursor += 1;
        Some(byte)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.cursor;
            let byte = self.next_byte()?;
            let kind = match byte {
                b'(' => TokenKind::LParen,
                b')' => TokenKind::RParen,
                b'{' => TokenKind::LBrace,
                b'}' => TokenKind::RBrace,
                b';' => TokenKind::Semicolon,
                b',' => TokenKind::Comma,
                b'.' => TokenKind::Dot,
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Minus,
                b'*' => TokenKind::Star,
                b'/' => {
                    if self.peek_byte::<0>() == Some(b'/') {
                        self.cursor += 1;
                        while self.next_byte()? != b'\n' {}
                        continue;
                    } else {
                        TokenKind::Slash
                    }
                }
                b'!' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    }
                }
                b'=' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    }
                }
                b'<' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    }
                }
                b'>' => {
                    if self.peek_byte::<0>() == Some(b'=') {
                        self.cursor += 1;
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
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
                    let lexeme = &self.source[start..self.cursor];
                    match Keyword::from_lexeme(lexeme) {
                        Some(keyword) => TokenKind::Keyword(keyword),
                        None => TokenKind::Identifier,
                    }
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

                    let lexeme = &self.source[start..self.cursor];
                    match Keyword::from_lexeme(lexeme) {
                        Some(keyword) => TokenKind::Keyword(keyword),
                        None => TokenKind::Identifier,
                    }
                }
                b'"' => loop {
                    match self.next_byte() {
                        None => {
                            let error = LexError::UnterminatedString;
                            return Some(Err(error));
                        }
                        Some(b'\\') => {
                            if self.next_byte().is_none() {
                                let error = LexError::UnterminatedString;
                                return Some(Err(error));
                            }
                        }
                        Some(b'"') => {
                            break TokenKind::String;
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
                    TokenKind::Number
                }
                0x00..=0x7F => {
                    let char = byte as char;
                    let end = self.cursor;
                    let range = Range { start, end };
                    let error = LexError::UnexpectedCharacter { char, range };
                    return Some(Err(error));
                }
            };
            let end = self.cursor;
            let range = Range { start, end };
            let token = Token { kind, range };
            return Some(Ok(token));
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { char: char, range: Range<usize> },
    UnterminatedString,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;

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
        let actual: Box<[TokenKind]> = lexer.map(Result::unwrap).map(|token| token.kind).collect();
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
        assert_eq!(&actual[..], &expected[..]);
    }
}
