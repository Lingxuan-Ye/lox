use core::range::Range;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Keyword(Keyword),

    Identifier,
    String,
    Number,
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
