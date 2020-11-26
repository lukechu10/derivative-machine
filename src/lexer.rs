use logos::Logos;

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    #[regex("[0-9.]+", |lex| lex.slice().parse())]
    Number(f64),
    #[regex("[a-zA-Z]+")]
    Identifier(&'a str),
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

impl<'a> Token<'a> {
    /// Returns the binding power for the binary operator or `(-1, -1)` if not a valid operator.
    pub fn get_bp(self) -> (i32, i32) {
        match self {
            Token::Plus | Token::Minus => (2, 1),
            Token::Asterisk | Token::Slash => (4, 3),
            Token::Exponent => (5, 6),
            _ => (-1, -1),
        }
    }
}
