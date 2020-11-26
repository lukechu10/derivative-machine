use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[regex("[0-9.]+", |lex| lex.slice().parse())]
    Number(f64),
    #[regex("[a-zA-Z]+", |lex| lex.slice().to_string())]
    Identifier(String),
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("^")]
    Exponent,
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl Token {
    /// Returns the binding power for the binary (infix) operator or `(-1, -1)` if not a valid operator.
    pub fn get_infix_bp(&self) -> (i32, i32) {
        match self {
            Token::Plus | Token::Minus => (1, 2),
            Token::Asterisk | Token::Slash => (3, 4),
            Token::Exponent => (6, 5), // right associative
            _ => (-1, -1),
        }
    }

    /// Returns the binding power for the prefix operator or `((), -1)` if not a valid operator.
    pub fn get_prefix_bp(&self) -> ((), i32) {
        match self {
            Token::Plus | Token::Minus => ((), 8),
            _ => ((), -1),
        }
    }
}
