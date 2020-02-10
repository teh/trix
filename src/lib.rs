use gc_arena::{
    make_arena, unsafe_empty_collect, ArenaParameters, Collect, Gc, GcCell, MutationContext,
};

type Environment<'a> = std::collections::HashMap<String, &'a Expr<'a>>;

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
///
pub enum Expr<'gc> {
    Int(i64),
    Bool(bool),
    String(&'gc str),
    Path(&'gc str),
    Null {},
    List {},
    Float {},
    Lambda {
        e: Gc<'gc, Expr<'gc>>,
        arg: Gc<'gc, Expr<'gc>>,
    },
    Attrs {},

    App {
        f: Gc<'gc, Expr<'gc>>,
        body: Gc<'gc, Expr<'gc>>,
    },
    PrimOp {
        name: &'gc str,
        arity: usize,
    },

    // partially applied primop
    PrimOpApp {
        chain: Gc<'gc, Expr<'gc>>,
        arity: usize,
    },
}

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
struct ExprRoot<'gc> {
    root: Gc<'gc, Expr<'gc>>,
}

make_arena!(ExprArena, ExprRoot);

fn eval<'gc, 'context>(
    mc: MutationContext<'gc, 'context>,
    expr: Gc<'gc, Expr<'gc>>,
) -> Gc<'gc, Expr<'gc>> {
    match *expr {
        // Expr::Int(_) => *self,
        Expr::App { f, body } => {
            let ef = eval(mc, f);
            match *ef {
                Expr::PrimOp { name, arity, .. } => Gc::allocate(mc, Expr::PrimOpApp {
                    arity: arity - 1,
                    chain: ef,
                }), // OK
                default => {
                    panic!("expected callable, got {:?}", ef);
                }
            }
        }
        // Expr::PrimOp { name, arity } => *self,
        // Expr::PrimOpApp { chain, arity } => panic!("invalid state"),
        default => {
            println!("default");
            expr
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // fn check_done() {
    //     let i = Expr::Int(10);
    //     let env = Environment::new();
    //     i.eval(&env);
    // }
    #[test]
    fn check_application() {
        let mut arena = ExprArena::new(ArenaParameters::default(), |mc| ExprRoot { root: Gc::allocate(mc, Expr::App {
            f: Gc::allocate(
                mc,
                Expr::App {
                    f: Gc::allocate(
                        mc,
                        Expr::PrimOp {
                            name: "plus",
                            arity: 2,
                        },
                    ),
                    body: Gc::allocate(mc, Expr::Int(2)),
                },
            ),
            body: Gc::allocate(mc, Expr::Int(1)),
        })});
        arena.mutate(|mc, root| {
            eval(mc, root.root);
        });
    }
}

// {env: ., (\x.\y.x + y) 1 2}
// {env: x=1, (\y.x + y) 2}
// {env: x=1, }
