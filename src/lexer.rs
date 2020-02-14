use crate::lexer::nix_lexer::{Lexer, Token, Error};
use lalrpop_util;
/// Unfortunately we can't get away with just using the lexer that ships with
/// lalrpop because it really is just a lexer, but string interpolation in nix
/// requires the lexer to switch between in-string and out-of-string scanning
/// mode. On top of that the lexer needs to keep track on how deep the
/// expression nesting is to make sure it's balanced.

pub mod nix_lexer {
    include!(concat!(env!("OUT_DIR"), "/nix_lexer.rs"));
}

#[derive(Debug)]
pub enum LexicalError {
    NotGood((usize, usize, usize))
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.yylex() {
            Ok(next_item) => {
                let length = self.yylength();
                let span = Ok((0, next_item, 1));
                Some(span)
            },
            Err(Error::EOF) => None,
            Err(Error::Unmatch) => Some(Err(LexicalError::NotGood(self.error_state()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_simple_lex() {
        let mut lexer = Lexer::new("1", Vec::new(), 0);
        assert_eq!(lexer.yylex().unwrap(), Token::INT(1));

        let mut lexer = Lexer::new("some_id", Vec::new(), 0);
        assert_eq!(lexer.yylex().unwrap(), Token::ID("some_id".to_string()));

        let mut lexer = Lexer::new("//", Vec::new(), 0);
        assert_eq!(lexer.yylex().unwrap(), Token::UPDATE);

        let mut lexer = Lexer::new("./path", Vec::new(), 0);
        assert_eq!(lexer.yylex().unwrap(), Token::PATH("./path".to_string()));

        let mut lexer = Lexer::new(r#""xx-s-xx""#, Vec::new(), 0);
        lexer.yylex().unwrap();
        assert_eq!(lexer.yylex().unwrap(), Token::STRING_PART("xx-s-xx".to_string()));

        let mut lexer = Lexer::new(r#""xx-s\\-xx""#, Vec::new(), 0);
        lexer.yylex().unwrap();
        assert_eq!(lexer.yylex().unwrap(), Token::STRING_PART(r"xx-s\\-xx".to_string()));
        // TODO escape sequences, interpolation
    }

    fn _collect(lexer: &mut Lexer) -> Vec<Token> {
        let mut ret = Vec::new();
        while let Ok(x) = lexer.yylex() {
            ret.push(x);
        }
        ret
    }

    #[test]
    fn check_nested() {
        let mut lexer = Lexer::new(r#""x${1}x""#, Vec::new(), 0);
        let vv = _collect(&mut lexer);
        println!("{:?}", vv);
        assert_eq!(vv, vec![
            Token::STRING_QUOTE,
            Token::STRING_PART("x".to_string()),
            Token::DOLLAR_CURLY,
            Token::INT(1),
            Token::CLOSE_CURLY,
            Token::STRING_PART("x".to_string()),
            Token::STRING_QUOTE,
        ]);
    }

    #[test]
    fn check_attr_lambda() {
        let s = include_str!("lang-tests/parse-okay-1.nix");
        let mut lexer = Lexer::new(s, Vec::new(), 0);

        let vv = _collect(&mut lexer);
        println!("{:?}", vv);
        assert_eq!(vv, vec![
            Token::OPEN_CURLY,
            Token::ID("x".to_string()),
            Token::COMMA,
            Token::ID("y".to_string()),
            Token::COMMA,
            Token::ID("z".to_string()),
            Token::CLOSE_CURLY,
            Token::COLON,
            Token::ID("x".to_string()),
            Token::PLUS,
            Token::ID("y".to_string()),
            Token::PLUS,
            Token::ID("z".to_string())
        ]);
    }

}
