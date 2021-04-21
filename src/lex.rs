#[derive(Debug)]
pub enum Token {
    LPar,
    RPar,
    Literal(String),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Literal(l1), Token::Literal(l2)) => l1 == l2,
            (Token::LPar, Token::LPar) | (Token::RPar, Token::RPar) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum LexError {
    UnknownToken(String),
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    input
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_ascii_whitespace()
        .map(|s| match s {
            "(" => Ok(Token::LPar),
            ")" => Ok(Token::RPar),
            _ => Ok(Token::Literal(s.to_string())),
        })
        .collect()
}
