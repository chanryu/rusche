use crate::{
    eval::{eval, EvalContext},
    lexer::tokenize,
    parser::{ParseError, Parser},
};

pub fn exec_src(src: &str, context: &EvalContext) -> Result<(), String> {
    let tokens = tokenize(src).map_err(|e| e.to_string())?;

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(None) => {
                break; // we're done!
            }
            Ok(Some(expr)) => {
                let _ = eval(&expr, context).map_err(|e| e.to_string())?;
            }
            Err(ParseError::NeedMoreToken) => {
                return Err(format!("Failed to parse source - incomplete expression"));
            }
            Err(ParseError::UnexpectedToken(token)) => {
                return Err(format!("Failed to parse source - unexpected token {token}"));
            }
        }
    }

    Ok(())
}
