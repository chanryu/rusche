use crate::span::{Loc, Span};
use crate::token::Token;
use std::iter::{Iterator, Peekable};

const TOKEN_DELIMITERS: &str = " \t\r\n()';\"";

#[derive(Debug, PartialEq)]
pub enum LexError {
    IncompleteString(Span),
    InvalidNumber(Span),
}

type LexResult = Result<Option<Token>, LexError>;

/// Lexical analyzer for the Rusche language.
pub struct Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    iter: Peekable<Iter>,
    loc: Loc,
}

impl<Iter> Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter, loc: Loc) -> Self {
        Self {
            iter: iter.peekable(),
            loc,
        }
    }

    /// Returns the next token from the input stream.
    pub fn get_token(&mut self) -> LexResult {
        loop {
            self.skip_spaces();
            if !self.skip_comment() {
                break;
            }
        }

        let begin_loc = self.loc;

        match self.next_char() {
            Some('(') => Ok(Some(Token::OpenParen(begin_loc))),
            Some(')') => Ok(Some(Token::CloseParen(begin_loc))),

            Some('\'') => Ok(Some(Token::Quote(begin_loc))),
            Some('`') => Ok(Some(Token::Quasiquote(begin_loc))),
            Some(',') => {
                if self.next_char_if(|ch| *ch == '@').is_some() {
                    Ok(Some(Token::UnquoteSplicing(begin_loc)))
                } else {
                    Ok(Some(Token::Unquote(begin_loc)))
                }
            }

            // string
            Some('"') => self.read_string(begin_loc),

            // number
            Some(ch) if ch.is_ascii_digit() => self.read_number(ch, begin_loc),

            // number or symbol
            Some(ch) if ch == '+' || ch == '-' => match self.iter.peek() {
                Some(&next_ch) if next_ch.is_ascii_digit() => self.read_number(ch, begin_loc),
                _ => self.read_symbol(ch, begin_loc),
            },

            // we allow all other characters to be a symbol
            Some(ch) => self.read_symbol(ch, begin_loc),

            None => Ok(None),
        }
    }

    fn skip_spaces(&mut self) {
        while self.next_char_if(|&ch| ch.is_whitespace()).is_some() {}
    }

    fn skip_comment(&mut self) -> bool {
        if self.iter.next_if_eq(&';').is_some() {
            let _ = self.iter.find(|&ch| ch == '\n');
            self.advance_loc(&Some('\n'));
            true
        } else {
            false
        }
    }

    fn read_string(&mut self, begin_loc: Loc) -> LexResult {
        let mut text = String::new();
        let mut escaped = false;
        while let Some(ch) = self.next_char() {
            match (ch, escaped) {
                ('\n', _) => return Err(LexError::IncompleteString(begin_loc.span_to(self.loc))),
                (ch, true) => {
                    escaped = false;
                    match ch {
                        'n' => text.push('\n'),
                        'r' => text.push('\r'),
                        't' => text.push('\t'),
                        _ => text.push(ch),
                    }
                }
                ('"', false) => return Ok(Some(Token::Str(text, begin_loc.span_to(self.loc)))),
                ('\\', false) => escaped = true,
                (ch, false) => text.push(ch),
            }
        }
        Err(LexError::IncompleteString(begin_loc.span_to(self.loc)))
    }

    fn read_number(&mut self, first_char: char, begin_loc: Loc) -> LexResult {
        let mut digits = String::new();

        if first_char.is_ascii_digit() {
            digits.push(first_char);
        }

        while let Some(ch) = self.next_char_if(|ch| !TOKEN_DELIMITERS.contains(*ch)) {
            digits.push(ch);
        }

        let sign = if first_char == '-' { -1.0 } else { 1.0 };
        let span = begin_loc.span_to(self.loc);

        digits
            .parse::<f64>()
            .map(|value| Some(Token::Num(value * sign, span)))
            .map_err(|_| LexError::InvalidNumber(span))
    }

    fn read_symbol(&mut self, first_char: char, begin_loc: Loc) -> LexResult {
        let mut name = String::with_capacity(16);
        name.push(first_char);

        while let Some(ch) = self.next_char_if(|ch| !TOKEN_DELIMITERS.contains(*ch)) {
            name.push(ch);
        }

        Ok(Some(Token::Sym(name, Span::new(begin_loc, self.loc))))
    }
}

impl<Iter> Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    fn next_char(&mut self) -> Option<char> {
        let ch = self.iter.next();
        self.advance_loc(&ch);
        ch
    }

    fn next_char_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        let ch = self.iter.next_if(func);
        self.advance_loc(&ch);
        ch
    }

    fn advance_loc(&mut self, ch: &Option<char>) {
        if let Some(ch) = ch {
            if *ch == '\n' {
                self.loc.line += 1;
                self.loc.column = 0;
            } else {
                self.loc.column += 1;
            }
        }
    }
}

/// A convinient function to tokenize a string. Internally, it uses the [`Lexer`] to tokenize
/// the input string.
pub fn tokenize(text: &str, loc: Loc) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(text.chars(), loc);

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
                assert_eq!($source.chars().next(), Some('"'));
                let chars = $source.chars();
                let token = Lexer::new(chars, Loc::default())
                    .get_token()
                    .unwrap()
                    .unwrap();
                assert_eq!(token, Token::Str(String::from($expected), token.span()));
            };
            ($source:literal, $expected:expr) => {
                assert_eq!($source.chars().next(), Some('"'));
                let chars = $source.chars();
                assert_eq!(
                    Lexer::new(chars, Loc::default()).get_token(),
                    Err($expected)
                );
            };
        }

        assert_parse_string!(r#""valid string""#, "valid string");
        assert_parse_string!(r#""an escaped\" string""#, "an escaped\" string");
        assert_parse_string!(
            r#""incomplete string"#,
            LexError::IncompleteString(Span::new(Loc::new(0, 0), Loc::new(0, 18)))
        );
    }

    #[test]
    fn test_read_number() {
        macro_rules! assert_parsed_number {
            ($source:literal, $expected:literal) => {
                assert!(!$source.is_empty());
                let chars = $source.chars();
                let token = Lexer::new(chars, Loc::default())
                    .get_token()
                    .unwrap()
                    .unwrap();
                assert_eq!(token, Token::Num($expected.into(), token.span()));
            };
        }

        assert_parsed_number!("0", 0);
        assert_parsed_number!("1", 1);
        assert_parsed_number!("1.1", 1.1);
        assert_parsed_number!("-1", -1);

        assert!(Lexer::new("123xya".chars(), Loc::default())
            .get_token()
            .is_err());
    }

    #[test]
    fn test_scanner_eof() {
        let mut lexer = Lexer::new("".chars(), Loc::default());
        assert_eq!(lexer.get_token(), Ok(None));

        let mut lexer = Lexer::new("    ".chars(), Loc::default());
        assert_eq!(lexer.get_token(), Ok(None));

        let mut lexer = Lexer::new("   ; comment".chars(), Loc::default());
        assert_eq!(lexer.get_token(), Ok(None));

        let mut lexer = Lexer::new("".chars(), Loc::default());
        assert_eq!(lexer.get_token(), Ok(None));
        assert_eq!(lexer.get_token(), Ok(None));
        assert_eq!(lexer.get_token(), Ok(None));
    }

    #[test]
    fn test_scanner_parans() {
        let mut lexer = Lexer::new("()(())(()())".chars(), Loc::default());
        macro_rules! match_next_paren {
            (None) => {
                assert_eq!(lexer.get_token().unwrap(), None);
            };
            ($token_case:ident) => {
                let token = lexer.get_token().unwrap().unwrap();
                let loc = Loc::new(1, 1); // don't care about the location
                assert_eq!(token, Token::$token_case(loc));
            };
        }

        match_next_paren!(OpenParen);
        match_next_paren!(CloseParen);
        match_next_paren!(OpenParen);
        match_next_paren!(OpenParen);
        match_next_paren!(CloseParen);
        match_next_paren!(CloseParen);
        match_next_paren!(OpenParen);
        match_next_paren!(OpenParen);
        match_next_paren!(CloseParen);
        match_next_paren!(OpenParen);
        match_next_paren!(CloseParen);
        match_next_paren!(CloseParen);
        match_next_paren!(None);
    }

    #[test]
    fn test_scanner_all_tokens() {
        let all_tokens = r#"
            ; comment
            (add 1 2.34 (x y) "test" '(100 200 300))
            ; another comment
        "#;

        let mut lexer = Lexer::new(all_tokens.chars(), Loc::default());
        macro_rules! match_next_token {
            (None) => {
                assert_eq!(lexer.get_token().unwrap(), None);
            };
            (Some($token_case:ident)) => {
                let token = lexer.get_token().unwrap().unwrap();
                let loc = Loc::new(1, 1); // don't care about the location
                assert_eq!(token, Token::$token_case(loc));
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

    #[test]
    fn test_span_factorial() {
        let src = r#"
            (define (factorial n)
                (if (= n 0)
                    1
                    (* n (factorial (- n 1)))))
        "#;

        let mut lexer = Lexer::new(src.chars(), Loc::default());

        macro_rules! match_next_span {
            ($span:expr) => {
                let token = lexer.get_token().unwrap().unwrap();
                assert_eq!($span, token.span());
            };
        }

        match_next_span!(Span::new(Loc::new(1, 12), Loc::new(1, 13))); // (
        match_next_span!(Span::new(Loc::new(1, 13), Loc::new(1, 19))); // define
        match_next_span!(Span::new(Loc::new(1, 20), Loc::new(1, 21))); // (
        match_next_span!(Span::new(Loc::new(1, 21), Loc::new(1, 30))); // factorial
        match_next_span!(Span::new(Loc::new(1, 31), Loc::new(1, 32))); // n
        match_next_span!(Span::new(Loc::new(1, 32), Loc::new(1, 33))); // )
        match_next_span!(Span::new(Loc::new(2, 16), Loc::new(2, 17))); // (
        match_next_span!(Span::new(Loc::new(2, 17), Loc::new(2, 19))); // if
        match_next_span!(Span::new(Loc::new(2, 20), Loc::new(2, 21))); // (
        match_next_span!(Span::new(Loc::new(2, 21), Loc::new(2, 22))); // =
        match_next_span!(Span::new(Loc::new(2, 23), Loc::new(2, 24))); // n
        match_next_span!(Span::new(Loc::new(2, 25), Loc::new(2, 26))); // 0
        match_next_span!(Span::new(Loc::new(2, 26), Loc::new(2, 27))); // )
        match_next_span!(Span::new(Loc::new(3, 20), Loc::new(3, 21))); // 1

        // ...
    }

    #[test]
    fn test_span_define_test() {
        let mut lexer = Lexer::new(r#"(define test "test")"#.chars(), Loc::default());

        macro_rules! match_next_span {
            (None) => {
                assert_eq!(lexer.get_token().unwrap(), None);
            };
            ($span:expr) => {
                let token = lexer.get_token().unwrap().unwrap();
                assert_eq!($span, token.span());
            };
        }

        match_next_span!(Span::new(Loc::new(0, 0), Loc::new(0, 1))); // (
        match_next_span!(Span::new(Loc::new(0, 1), Loc::new(0, 7))); // define
        match_next_span!(Span::new(Loc::new(0, 8), Loc::new(0, 12))); // test
        match_next_span!(Span::new(Loc::new(0, 13), Loc::new(0, 19))); // "test"
        match_next_span!(Span::new(Loc::new(0, 19), Loc::new(0, 20))); // )
        match_next_span!(None);
    }
}
