/// Unfortunately we can't get away with just using the lexer that ships with
/// lalrpop because it really is just a lexer, but string interpolation in nix
/// requires the lexer to switch between in-string and out-of-string scanning
/// mode. On top of that the lexer needs to keep track on how deep the
/// expression nesting is to make sure it's balanced.
use std::str::CharIndices;
use regex::{RegexSet};
use lalrpop_util;

pub struct Lexer<'input> {
    // we need to keep track how need we are within the string-interpolation
    // stack.
    text: &'input str,
    interpolation_stack: u32,
    consumed: usize,
}

pub enum LexicalError {
}

// macro ensures that regexes and enum values line up exactly
macro_rules! MakeTokens {
    ( $( $name:ident => $x:expr ),* ) => (
            #[derive(PartialEq, Eq, Clone, Debug)]
            pub enum Tok<'input> {
            $(
                $name(&'input str),
            )*
            }

            lazy_static!{
                static ref token_set: RegexSet = RegexSet::new(&[
                $(
                    $x,
                )*
                ]);
            }
        )
}

MakeTokens!(
    ID => r"",
    INT => r"",
    FLOAT => r"",
    STRING_START => r"",
    DOLLAR_CURLY => r"",
    PATH => r"/[^/]+",
    // keywords
    IF => "if",
    THEN => "then",
    ELSE => "else",
    ASSERT => "assert",
    WITH => "with",
    LET => "let",
    IN => "in",
    REC => "rec",
    INHERIT => "inherit",
    OR_KW => "or",
    ELLIPSIS => "...",
    // operators
    EQ => "==",
    NEQ => "!=",
    LEQ => "<=",
    GEQ => ">=",
    AND => "&&",
    LT => "<",
    GT => ">",
    OR => "||",
    IMPL => "->",
    UPDATE => "//",
    CONCAT => "++",
    MINUS => "-",
    PLUS => "+",
    DIVIDE => "/",
    MULTIPLY => "*",
    ASSIGN => "=",
    // other
    COMMA => ",",
    DOT => ".",
    COLON => ":",
    SEMICOLON => ";",
    QUESTIONMARK => "?",
    AT => "@",
    NEGATE => "!",

    OPEN_CURLY => "{",
    CLOSE_CURLY => "}",
    OPEN_PAREN => "(",
    CLOSE_PAREN => ")",
    OPEN_SQUARE => "[",
    CLOSE_SQUARE => "]"
);

impl<'input> Lexer<'input> {
    pub fn new(text: &'input str) -> Self {
        Lexer {
            text,
            consumed: 0,
            interpolation_stack: 0,
        }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Tok, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        // We always want to trim whitespace, _unless_ we  are in a string-situation. E.g.
        // for "x${"x"}x" to expand to "xxx" we can't skip whitespace after the
        // expression-closing "}".
        let __text = self.text.trim_start();
        let __whitespace = self.text.len() - __text.len();
        let __start_offset = self.consumed + __whitespace;
        if __text.is_empty() {
            self.text = __text;
            self.consumed = __start_offset;
            None
        } else {
            let __matches = token_set.matches(__text);
            if !__matches.matched_any() {
                Some(Err(lalrpop_util::ParseError::InvalidToken {
                    location: __start_offset,
                }))
            } else {
                let mut __longest_match = 0;
                let mut __index = 0;
                for __i in 0..43 {
                    if __matches.matched(__i) {
                        let __match = self.regex_vec[__i].find(__text).unwrap();
                        let __len = __match.end();
                        if __len >= __longest_match {
                            __longest_match = __len;
                            __index = __i;
                        }
                    }
                }
                let __result = &__text[..__longest_match];
                let __remaining = &__text[__longest_match..];
                let __end_offset = __start_offset + __longest_match;
                self.text = __remaining;
                self.consumed = __end_offset;
                Some(Ok((__start_offset, Tok(__index, __result), __end_offset)))
            }
        }
    }
}
