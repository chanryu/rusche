use rusp::{
    lexer::{LexError, Lexer},
    token::Token,
};

pub fn tokenize(text: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(text.chars());

    while let Some(token) = lexer.get_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}
