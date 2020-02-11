use crate::expr::{Cont, Env, Expr, GcEnv, GcExpr, GcStack};
use gc_arena::{MutationContext, Gc,  rootless_arena};
use nom::bytes::complete::{is_not, tag, take};
use nom::sequence::delimited;
use nom::{alt, character::complete::char, map, IResult};

fn pexpr<'gc>(mc: MutationContext<'gc, '_>, i: &'gc str) -> IResult<&'gc str, GcExpr<'gc>> {
    return Ok((i, Gc::allocate(mc, Expr::Int(10))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_simple_expression() {
        rootless_arena(|mc| {
            let s = "";
            let (_, result) = pexpr(mc, s).unwrap();
            assert_eq!(*result, Expr::Int(10));
        });
    }
}
