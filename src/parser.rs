
#[cfg(test)]
mod tests {
    use super::*;
    use gc_arena::{rootless_arena};
    use crate::lexer::nix_lexer::Lexer;
    #[test]
    fn check_simple_expression() {
        rootless_arena(|mc| {
            // TODO newlines are broken
            let s = include_str!("lang-tests/parse-okay-1.nix");
            let mut lex = Lexer::new(s, Vec::new(), 0);

            let i = crate::expr_parser::exprParser::new().parse(mc, lex);
            println!("{:?}", *i.unwrap());
        });
    }
    // #[test]
    // fn check_string() {
    //     rootless_arena(|mc| {
    //         let i = crate::expr_parser::exprParser::new().parse(mc, "\"hello\"").unwrap();
    //         println!("{:?}", *i);
    //         let i = crate::expr_parser::exprParser::new().parse(mc, "\"hello ${1}\"").unwrap();
    //         println!("{:?}", *i);
    //     });
    // }
}
