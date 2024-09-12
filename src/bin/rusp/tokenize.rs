use rusp::{
    scanner::{Scanner, TokenError},
    token::Token,
};

pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenError> {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(text.chars());

    loop {
        match scanner.get_token()? {
            Some(token) => tokens.push(token),
            None => return Ok(tokens),
        }
    }
}
