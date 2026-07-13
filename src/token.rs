use core::fmt;
use core::range::Range;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Range<usize>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = &self.kind;
        let Range { start, end } = self.range;
        write!(f, "`{kind}` at {start}..{end}")
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Identifier,
    String,
    Number,

    Keyword(Keyword),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::Semicolon => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Bang => write!(f, "!"),
            Self::BangEqual => write!(f, "!="),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => write!(f, "=="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Identifier => write!(f, "identifier"),
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Keyword(keyword) => write!(f, "{keyword}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Print,
    Class,
    Super,
    Fun,
    This,
    Var,
    While,
    For,
    If,
    Else,
    And,
    Or,
    True,
    False,
    Nil,
    Return,
}

impl Keyword {
    pub fn from_lexeme(lexeme: &str) -> Option<Self> {
        match lexeme {
            "print" => Some(Self::Print),
            "class" => Some(Self::Class),
            "super" => Some(Self::Super),
            "fun" => Some(Self::Fun),
            "this" => Some(Self::This),
            "var" => Some(Self::Var),
            "while" => Some(Self::While),
            "for" => Some(Self::For),
            "if" => Some(Self::If),
            "else" => Some(Self::Else),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "true" => Some(Self::True),
            "false" => Some(Self::False),
            "nil" => Some(Self::Nil),
            "return" => Some(Self::Return),
            _ => None,
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Print => write!(f, "print"),
            Self::Class => write!(f, "class"),
            Self::Super => write!(f, "super"),
            Self::Fun => write!(f, "fun"),
            Self::This => write!(f, "this"),
            Self::Var => write!(f, "var"),
            Self::While => write!(f, "while"),
            Self::For => write!(f, "for"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Nil => write!(f, "nil"),
            Self::Return => write!(f, "return"),
        }
    }
}
