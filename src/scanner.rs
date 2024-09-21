use crate::token::{Loc, Token};
use std::fmt;
use std::iter::{Iterator, Peekable};

const SYMBOL_DELIMITERS: &str = " \t\r\n()';\"";

#[derive(Debug, PartialEq)]
pub enum TokenError {
    IncompleteString,
    InvalidNumber,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::IncompleteString => write!(f, "Incomplete string"),
            TokenError::InvalidNumber => write!(f, "Invalid number"),
        }
    }
}

type ScanResult = Result<Option<Token>, TokenError>;

pub struct Scanner<Iter>
where
    Iter: Iterator<Item = char>,
{
    __iter: Peekable<Iter>,
    loc: Loc,
}

impl<Iter> Scanner<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter) -> Self {
        Self {
            __iter: iter.peekable(),
            loc: Loc::new(1, 0),
        }
    }

    pub fn get_token(&mut self) -> ScanResult {
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

    fn read_string(&mut self) -> ScanResult {
        let loc = self.loc;
        let mut text = String::new();
        let mut escaped = false;
        while let Some(ch) = self.next_char() {
            match (ch, escaped) {
                ('\n', _) => return Err(TokenError::IncompleteString),
                (ch, true) => {
                    escaped = false;
                    match ch {
                        'n' => text.push('\n'),
                        'r' => text.push('\r'),
                        't' => text.push('\t'),
                        _ => text.push(ch),
                    }
                }
                ('"', false) => return Ok(Some(Token::Str(loc, text))),
                ('\\', false) => escaped = true,
                (ch, false) => text.push(ch),
            }
        }
        Err(TokenError::IncompleteString)
    }

    fn read_number(&mut self, first_char: char, sign: i32) -> ScanResult {
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
            .map(|value| Some(Token::Num(loc, value * sign as f64)))
            .map_err(|_| TokenError::InvalidNumber)
    }

    fn read_symbol(&mut self, first_char: char) -> ScanResult {
        let loc = self.loc;
        let mut name = String::with_capacity(16);
        name.push(first_char);

        while let Some(ch) = self.next_char_if(|ch| !SYMBOL_DELIMITERS.contains(*ch)) {
            name.push(ch);
        }

        Ok(Some(Token::Sym(loc, name)))
    }
}

impl<Iter> Scanner<Iter>
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

#[cfg(test)]
mod tests {
    use crate::token::Loc;

    use super::*;

    fn num<T>(value: T) -> Token
    where
        T: Into<f64>,
    {
        Token::Num(Loc::new(1, 0), value.into())
    }

    #[test]
    fn test_read_string() {
        macro_rules! parse_string_assert_eq {
            ($source:literal, $expected:expr) => {
                let mut chars = $source.chars();
                assert_eq!(chars.next().unwrap(), '"');
                assert_eq!(Scanner::new(chars).read_string(), $expected);
            };
        }

        let loc = Loc::new(1, 0);

        parse_string_assert_eq!(
            r#""valid string""#,
            Ok(Some(Token::Str(loc, "valid string".into())))
        );
        parse_string_assert_eq!(
            r#""an escaped\" string""#,
            Ok(Some(Token::Str(loc, String::from("an escaped\" string"))))
        );
        parse_string_assert_eq!(r#""incomplete string"#, Err(TokenError::IncompleteString));
    }

    #[test]
    fn test_read_number() {
        macro_rules! parse_number_assert_eq {
            ($source:literal, $expected:expr) => {
                assert!(!$source.is_empty());
                let mut chars = $source.chars();
                let first_char = chars.next().unwrap();
                assert_eq!(Scanner::new(chars).read_number(first_char, 1), $expected);
            };
        }

        parse_number_assert_eq!("0", Ok(Some(num(0))));
        parse_number_assert_eq!("1", Ok(Some(num(1))));
        parse_number_assert_eq!("1.1", Ok(Some(num(1.1))));
        parse_number_assert_eq!("-1", Ok(Some(num(-1))));
    }

    #[test]
    fn test_scanner_eof() {
        let mut scanner = Scanner::new("".chars());
        assert_eq!(scanner.get_token(), Ok(None));

        let mut scanner = Scanner::new("    ".chars());
        assert_eq!(scanner.get_token(), Ok(None));

        let mut scanner = Scanner::new("   ; comment".chars());
        assert_eq!(scanner.get_token(), Ok(None));

        let mut scanner = Scanner::new("".chars());
        assert_eq!(scanner.get_token(), Ok(None));
        assert_eq!(scanner.get_token(), Ok(None));
        assert_eq!(scanner.get_token(), Ok(None));
    }

    #[test]
    fn test_scanner_parans() {
        let mut scanner = Scanner::new("()(())(()())".chars());
        macro_rules! match_next_paran {
            (None) => {
                assert_eq!(scanner.get_token().unwrap(), None);
            };
            (Some($token_case:ident)) => {
                let token = scanner.get_token().unwrap().unwrap();
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

        let mut scanner = Scanner::new(all_tokens.chars());
        macro_rules! match_next_token {
            (None) => {
                assert_eq!(scanner.get_token().unwrap(), None);
            };
            (Some($token_case:ident)) => {
                let token = scanner.get_token().unwrap().unwrap();
                assert_eq!(token, Token::$token_case(token.loc()));
            };
            (Some($token_case:ident($value:expr))) => {
                let token = scanner.get_token().unwrap().unwrap();
                assert_eq!(token, Token::$token_case(token.loc(), $value));
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
