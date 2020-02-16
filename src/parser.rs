#[cfg(test)]
mod tests {
    use crate::lexer::nix_lexer::Lexer;
    use gc_arena::rootless_arena;
    use crate::expr_parser::exprParser;

    #[test]
    fn check_simple_expression() {
        rootless_arena(|mc| {
            // TODO newlines are broken
            let s = include_str!("lang-tests/parse-okay-1.nix");
            let lex = Lexer::new(s, Vec::new(), 0);

            let i = crate::expr_parser::exprParser::new().parse(mc, lex);
            println!("{:?}", *i.unwrap());
        });
    }

    #[test]
    fn parse_nested_lambda() {
        let mut lexer = Lexer::new(
            "let f = x: y: x || y; in f",
            Vec::with_capacity(10),
            0,
        );
        rootless_arena(|mc| match crate::expr_parser::exprParser::new().parse(mc, lexer) {
            Ok(i) => println!("{:?}", *i),
            Err(err) => panic!("invalid parse: {:?}", err),
        });
    }

    #[test]
    fn parse_autoargs() {
        let mut lexer = Lexer::new(
            "let x = 1; in { a ? 1, b ? 2}: a + b",
            Vec::with_capacity(10),
            0,
        );
        rootless_arena(|mc| match crate::expr_parser::exprParser::new().parse(mc, lexer) {
            Ok(i) => println!("{:?}", *i),
            Err(err) => panic!("invalid parse: {:?}", err),
        });
    }

    #[test]
    fn parse_lambda_head() {
        let mut lexer = Lexer::new(
            "f {}",
            Vec::with_capacity(10),
            0,
        );
        rootless_arena(|mc| match crate::expr_parser::exprParser::new().parse(mc, lexer) {
            Ok(i) => println!("{:?}", *i),
            Err(err) => panic!("invalid parse: {:?}", err),
        });
    }

    #[test]
    fn parse_inherit() {
        let mut lexer = Lexer::new(
            "{ inherit pkgs; }",
            Vec::with_capacity(10),
            0,
        );
        rootless_arena(|mc| match crate::expr_parser::exprParser::new().parse(mc, lexer) {
            Ok(i) => println!("{:?}", *i),
            Err(err) => panic!("invalid parse: {:?}", err),
        });
    }

    #[test]
    fn smoke_test_parsing() {
        // lex all the files that we also expect to parse OK
        let m1 = glob::glob("./src/lang-tests/parse-okay-*.nix").expect("invalid glob pattern");
        let m2 = glob::glob("./src/lang-tests/eval-okay-*.nix").expect("invalid glob pattern");
        let m3 = m1.chain(m2);
        for entry in m3 {
            match entry {
                Ok(path) => {
                    println!("{:?}", path);
                    let s = std::fs::read_to_string(path).expect("could not read file");
                    let mut lexer = Lexer::new(&s, Vec::with_capacity(10), 0);
                    rootless_arena(|mc| match crate::expr_parser::exprParser::new().parse(mc, lexer) {
                        Ok(i) => println!("{:?}", i),
                        Err(err) => panic!("invalid parse: {:?}", err),
                    });
                }
                Err(e) => {
                    panic!("not a valid file: {:?}", e);
                }
            }
        }
    }
}
