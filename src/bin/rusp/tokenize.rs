use rusp::{
    scanner::{Scanner, TokenError},
    token::Token,
};

pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenError> {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(text.chars());

    while let Some(token) = scanner.get_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}
