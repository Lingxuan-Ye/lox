use std::range::Range;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub range: Range<usize>,
    pub kind: TokenKind,
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

#[derive(Debug, PartialEq)]
pub enum Keyword {
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
    Print,
    Return,
}

impl Keyword {
    pub fn from_lexeme(lexeme: &str) -> Option<Self> {
        match lexeme {
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
            "print" => Some(Self::Print),
            "return" => Some(Self::Return),
            _ => None,
        }
    }
}
