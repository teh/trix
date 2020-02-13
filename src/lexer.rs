use crate::lexer::nix_lexer::{Lexer, Token};
use lalrpop_util;
/// Unfortunately we can't get away with just using the lexer that ships with
/// lalrpop because it really is just a lexer, but string interpolation in nix
/// requires the lexer to switch between in-string and out-of-string scanning
/// mode. On top of that the lexer needs to keep track on how deep the
/// expression nesting is to make sure it's balanced.

pub mod nix_lexer {
    include!(concat!(env!("OUT_DIR"), "/nix_lexer.rs"));
}

pub enum LexicalError {
    NotGood
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.yylex() {
            Ok(next_item) => {
                let length = self.yylength();
                let span = Ok((0, next_item, 1));
                Some(span)
            }
            error => Some(Err(LexicalError::NotGood)),
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
        println!("{:?}", lexer.yylex());
    }
}
