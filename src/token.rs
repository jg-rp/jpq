use core::fmt;

pub const EOQ: char = '\0';

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Eoq,
    Error { msg: Box<str> },

    Colon,
    Comma,
    DoubleDot,
    Filter,
    Index { value: Box<str> },
    Key,  // non-standard
    Keys, // non-standard
    LBracket,
    Name { value: Box<str> },
    RBracket,
    Root,
    Wild,

    And,
    Current,
    DoubleQuoteString { value: Box<str> },
    Eq,
    False,
    Float { value: Box<str> },
    Function { name: Box<str> },
    Ge,
    Gt,
    Int { value: Box<str> },
    Le,
    LParen,
    Lt,
    Ne,
    Not,
    Null,
    Or,
    RParen,
    SingleQuoteString { value: Box<str> },
    True,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenType::*;
        match self {
            Eoq => f.write_str("`end of query`"),
            Error { msg } => write!(f, "error: {}", *msg),
            Colon => f.write_str("`;`"),
            Comma => f.write_str("`,`"),
            DoubleDot => f.write_str("`..`"),
            Filter => f.write_str("`?`"),
            Index { value } => write!(f, "`{}`", *value),
            Key => f.write_str("`#`"),
            Keys => f.write_str("`~`"),
            LBracket => f.write_str("`[`"),
            Name { value } => write!(f, "`{}`", *value),
            RBracket => f.write_str("`]`"),
            Root => f.write_str("`$`"),
            Wild => f.write_str("`*`"),
            And => f.write_str("`&&`"),
            Current => f.write_str("`@`"),
            DoubleQuoteString { value } => write!(f, "`{}`", *value),
            Eq => f.write_str("`==`"),
            False => f.write_str("`false`"),
            Float { value } => write!(f, "{}", *value),
            Function { name } => write!(f, "`{}`", *name),
            Ge => f.write_str("`>=`"),
            Gt => f.write_str("`>`"),
            Int { value } => write!(f, "{}", *value),
            Le => f.write_str("<=`"),
            LParen => f.write_str("`(`"),
            Lt => f.write_str("`<`"),
            Ne => f.write_str("`!=`"),
            Not => f.write_str("`!`"),
            Null => f.write_str("`null`"),
            Or => f.write_str("`or`"),
            RParen => f.write_str("`)`"),
            SingleQuoteString { value } => write!(f, "`{}`", *value),
            True => f.write_str("`true`"),
        }
    }
}

/// A JSONPath expression token, as produced by the lexer.
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub span: (usize, usize),
}

impl Token {
    pub fn new(kind: TokenType, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: (start, end),
        }
    }
}
