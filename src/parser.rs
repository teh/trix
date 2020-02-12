
#[cfg(test)]
mod tests {
    use super::*;
    use gc_arena::{rootless_arena};
    #[test]
    fn check_simple_expression() {
        rootless_arena(|mc| {
            let s = include_str!("lang-tests/parse-okay-1.nix");
            let i = crate::expr_parser::exprParser::new().parse(mc, "{ ... }: 1").unwrap();
            println!("{:?}", *i);
        });
    }
}
