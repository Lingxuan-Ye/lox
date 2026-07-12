use std::error::Error;
use std::fmt;
use std::range::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerError<'a> {
    UnexpectedCharacter {
        range: Range<usize>,
        char: char,
    },
    UnterminatedString,
    InvalidEscapeSequence {
        range: Range<usize>,
        sequence: &'a str,
    },
}

impl fmt::Display for LexerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { range, char } => {
                let Range { start, end } = range;
                write!(f, "unexpected character '{char}' at {start}..{end}")
            }
            Self::UnterminatedString => {
                write!(f, "unterminated string")
            }
            Self::InvalidEscapeSequence { range, sequence } => {
                let Range { start, end } = range;
                write!(f, "invalid escape sequence '{sequence}' at {start}..{end}")
            }
        }
    }
}

impl Error for LexerError<'_> {}
