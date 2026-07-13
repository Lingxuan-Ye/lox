use core::error::Error;
use core::fmt;
use core::range::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerError {
    UnexpectedCharacter { range: Range<usize>, char: char },
    UnterminatedString,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { range, char } => {
                let Range { start, end } = range;
                write!(f, "unexpected character '{char}' at {start}..{end}")
            }
            Self::UnterminatedString => {
                write!(f, "unterminated string")
            }
        }
    }
}

impl Error for LexerError {}
