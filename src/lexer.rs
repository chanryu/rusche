use crate::span::{Loc, Span};
use crate::token::Token;
use std::fmt;
use std::iter::{Iterator, Peekable};

const SYMBOL_DELIMITERS: &str = " \t\r\n()';\"";

#[derive(Debug, PartialEq)]
pub enum LexError {
    IncompleteString,
    InvalidNumber,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::IncompleteString => write!(f, "Incomplete string"),
            LexError::InvalidNumber => write!(f, "Invalid number"),
        }
    }
}

type LexResult = Result<Option<Token>, LexError>;

pub struct Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    __iter: Peekable<Iter>,
    loc: Loc,
}

impl<Iter> Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter) -> Self {
        Self {
            __iter: iter.peekable(),
            loc: Loc::new(1, 0),
        }
    }

    pub fn get_token(&mut self) -> LexResult {
        loop {
            self.skip_spaces();
            if !self.skip_comment() {
                break;
            }
        }

        match self.next_char() {
            Some('(') => Ok(Some(Token::OpenParen(self.loc))),
            Some(')') => Ok(Some(Token::CloseParen(self.loc))),

            Some('\'') => Ok(Some(Token::Quote(self.loc))),
            Some('`') => Ok(Some(Token::Quasiquote(self.loc))),
            Some(',') => {
                if self.next_char_if(|ch| *ch == '@').is_some() {
                    Ok(Some(Token::UnquoteSplicing(self.loc)))
                } else {
                    Ok(Some(Token::Unquote(self.loc)))
                }
            }

            // string
            Some('"') => self.read_string(),

            // number
            Some(ch) if ch.is_ascii_digit() || ch == '.' => self.read_number(ch, 1),

            // number or symbol
            Some('-') => {
                if let Some(ch) = self.next_char_if(|ch| ch.is_ascii_digit()) {
                    self.read_number(ch, -1)
                } else {
                    self.read_symbol('-')
                }
            }

            // we allow all other characters to be a symbol
            Some(ch) => self.read_symbol(ch),

            None => Ok(None),
        }
    }

    fn skip_spaces(&mut self) {
        while self.next_char_if(|&ch| ch.is_whitespace()).is_some() {}
    }

    fn skip_comment(&mut self) -> bool {
        if self.__iter.next_if_eq(&';').is_some() {
            let _ = self.__iter.find(|&ch| ch == '\n');
            self.advance_loc(&Some('\n'));
            true
        } else {
            false
        }
    }

    fn read_string(&mut self) -> LexResult {
        let loc = self.loc;
        let mut text = String::new();
        let mut escaped = false;
        while let Some(ch) = self.next_char() {
            match (ch, escaped) {
                ('\n', _) => return Err(LexError::IncompleteString),
                (ch, true) => {
                    escaped = false;
                    match ch {
                        'n' => text.push('\n'),
                        'r' => text.push('\r'),
                        't' => text.push('\t'),
                        _ => text.push(ch),
                    }
                }
                ('"', false) => return Ok(Some(Token::Str(text, Span::new(loc, self.loc)))),
                ('\\', false) => escaped = true,
                (ch, false) => text.push(ch),
            }
        }
        Err(LexError::IncompleteString)
    }

    fn read_number(&mut self, first_char: char, sign: i32) -> LexResult {
        let loc = self.loc;
        let mut has_decimal_point = first_char == '.';
        let mut digits = String::new();

        digits.push(first_char);
        while let Some(ch) =
            self.next_char_if(|&ch| ch.is_ascii_digit() || (!has_decimal_point && ch == '.'))
        {
            digits.push(ch);
            if ch == '.' {
                has_decimal_point = true;
            }
        }

        digits
            .parse::<f64>()
            .map(|value| Some(Token::Num(value * sign as f64, Span::new(loc, self.loc))))
            .map_err(|_| LexError::InvalidNumber)
    }

    fn read_symbol(&mut self, first_char: char) -> LexResult {
        let loc = self.loc;
        let mut name = String::with_capacity(16);
        name.push(first_char);

        while let Some(ch) = self.next_char_if(|ch| !SYMBOL_DELIMITERS.contains(*ch)) {
            name.push(ch);
        }

        Ok(Some(Token::Sym(name, Span::new(loc, self.loc))))
    }
}

impl<Iter> Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    fn next_char(&mut self) -> Option<char> {
        let ch = self.__iter.next();
        self.advance_loc(&ch);
        ch
    }

    fn next_char_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        let ch = self.__iter.next_if(func);
        self.advance_loc(&ch);
        ch
    }

    fn advance_loc(&mut self, ch: &Option<char>) {
        if let Some(ch) = ch {
            if *ch == '\n' {
                self.loc.line += 1;
                self.loc.column = 1;
            } else {
                self.loc.column += 1;
            }
        }
    }
}

pub fn tokenize(text: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(text.chars());

    while let Some(token) = lexer.get_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string() {
        macro_rules! assert_parse_string {
            ($source:literal, $expected:literal) => {
                let mut chars = $source.chars();
                assert_eq!(chars.next().unwrap(), '"');
                let token = Lexer::new(chars).read_string().unwrap().unwrap();
                assert_eq!(token, Token::Str(String::from($expected), token.span()));
            };
            ($source:literal, $expected:ident) => {
                let mut chars = $source.chars();
                assert_eq!(chars.next().unwrap(), '"');
                assert_eq!(Lexer::new(chars).read_string(), Err(LexError::$expected));
            };
        }

        assert_parse_string!(r#""valid string""#, "valid string");
        assert_parse_string!(r#""an escaped\" string""#, "an escaped\" string");
        assert_parse_string!(r#""incomplete string"#, IncompleteString);
    }

    #[test]
    fn test_read_number() {
        macro_rules! assert_parsed_number {
            ($source:literal, $expected:literal) => {
                assert!(!$source.is_empty());
                let mut chars = $source.chars();
                let first_char = chars.next().unwrap();
                let token = Lexer::new(chars)
                    .read_number(first_char, 1)
                    .unwrap()
                    .unwrap();
                assert_eq!(token, Token::Num($expected.into(), token.span()));
            };
        }

        assert_parsed_number!("0", 0);
        assert_parsed_number!("1", 1);
        assert_parsed_number!("1.1", 1.1);
        assert_parsed_number!("-1", -1);
    }

    #[test]
    fn test_scanner_eof() {
        let mut lexer = Lexer::new("".chars());
        assert_eq!(lexer.get_token(), Ok(None));

        let mut lexer = Lexer::new("    ".chars());
        assert_eq!(lexer.get_token(), Ok(None));

        let mut lexer = Lexer::new("   ; comment".chars());
        assert_eq!(lexer.get_token(), Ok(None));

        let mut lexer = Lexer::new("".chars());
        assert_eq!(lexer.get_token(), Ok(None));
        assert_eq!(lexer.get_token(), Ok(None));
        assert_eq!(lexer.get_token(), Ok(None));
    }

    #[test]
    fn test_scanner_parans() {
        let mut lexer = Lexer::new("()(())(()())".chars());
        macro_rules! match_next_paran {
            (None) => {
                assert_eq!(lexer.get_token().unwrap(), None);
            };
            (Some($token_case:ident)) => {
                let token = lexer.get_token().unwrap().unwrap();
                assert_eq!(token, Token::$token_case(token.loc()));
            };
        }

        match_next_paran!(Some(OpenParen));
        match_next_paran!(Some(CloseParen));
        match_next_paran!(Some(OpenParen));
        match_next_paran!(Some(OpenParen));
        match_next_paran!(Some(CloseParen));
        match_next_paran!(Some(CloseParen));
        match_next_paran!(Some(OpenParen));
        match_next_paran!(Some(OpenParen));
        match_next_paran!(Some(CloseParen));
        match_next_paran!(Some(OpenParen));
        match_next_paran!(Some(CloseParen));
        match_next_paran!(Some(CloseParen));
        match_next_paran!(None);
    }

    #[test]
    fn test_scanner_all_tokens() {
        let all_tokens = r#"
            ; comment
            (add 1 2.34 (x y) "test" '(100 200 300))
            ; another comment
        "#;

        let mut lexer = Lexer::new(all_tokens.chars());
        macro_rules! match_next_token {
            (None) => {
                assert_eq!(lexer.get_token().unwrap(), None);
            };
            (Some($token_case:ident)) => {
                let token = lexer.get_token().unwrap().unwrap();
                assert_eq!(token, Token::$token_case(token.loc()));
            };
            (Some($token_case:ident($value:expr))) => {
                let token = lexer.get_token().unwrap().unwrap();
                assert_eq!(token, Token::$token_case($value, token.span()));
            };
        }

        match_next_token!(Some(OpenParen));
        match_next_token!(Some(Sym("add".into())));
        match_next_token!(Some(Num(1.0)));
        match_next_token!(Some(Num(2.34)));
        match_next_token!(Some(OpenParen));
        match_next_token!(Some(Sym("x".into())));
        match_next_token!(Some(Sym("y".into())));
        match_next_token!(Some(CloseParen));
        match_next_token!(Some(Str("test".into())));
        match_next_token!(Some(Quote));
        match_next_token!(Some(OpenParen));
        match_next_token!(Some(Num(100.0)));
        match_next_token!(Some(Num(200.0)));
        match_next_token!(Some(Num(300.0)));
        match_next_token!(Some(CloseParen));
        match_next_token!(Some(CloseParen));
        match_next_token!(None);
    }
}
