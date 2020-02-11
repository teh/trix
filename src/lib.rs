use gc_arena::{make_arena, ArenaParameters, Collect, Gc, GcCell, MutationContext};
use std::collections::HashMap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
///
pub enum Expr<'gc> {
    Null(),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(&'gc str),
    Path(&'gc str),
    List(Vec<GcExpr<'gc>>),
    Attrs {},
    Lambda {
        e: GcExpr<'gc>,
    },
    App {
        f: GcExpr<'gc>,
        args: Vec<GcExpr<'gc>>,
        arity: usize,
    },
    Pap {
        f: GcExpr<'gc>,
        args: Vec<GcExpr<'gc>>,
        arity: usize,
    },
    Thunk {
        t: GcExpr<'gc>,
    },
    PrimOp {
        name: &'gc str,
        arity: usize,
    },
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct Env<'gc> {
    up: Option<Gc<'gc, Env<'gc>>>,
    values: HashMap<String, GcExpr<'gc>>,
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub enum Cont<'gc> {
    ApplyCont {
        args: Vec<GcExpr<'gc>>,
        env: GcEnv<'gc>,
        arity: usize,
    },
}

impl<'gc> Env<'gc> {
    fn new_root() -> Env<'gc> {
        Env {
            up: None,
            values: HashMap::new(),
        }
    }
}

type GcExpr<'gc> = Gc<'gc, Expr<'gc>>;
type GcEnv<'gc> = Gc<'gc, Env<'gc>>;
type GcStack<'gc> = GcCell<'gc, Vec<Cont<'gc>>>;

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
struct ExprRoot<'gc> {
    root: GcExpr<'gc>,
    env: GcEnv<'gc>,
    stack: GcStack<'gc>,
}

make_arena!(ExprArena, ExprRoot);

fn step<'gc>(
    mc: MutationContext<'gc, '_>,
    expr: GcExpr<'gc>,
    env: GcEnv<'gc>,
    black_hole: GcExpr<'gc>,
    stack: GcStack<'gc>,
) -> (GcExpr<'gc>, GcEnv<'gc>) {
    println!("step {:?}", *expr);

    if Gc::ptr_eq(expr, black_hole) {
        panic!("found black_hole, probably infinite recursion");
    }

    let stack_top = stack.read().last().cloned();
    match (&*expr, &stack_top) {
        // (Expr::Let { bindings, in_ }, _) => {
        // create chained environment, return (env, in_); any expression in
        // a let binding becomes a thunk with an update, when stepping into
        // we push UpdateCont for the allocation, and when value +
        // UpdateCont we update.

        // NB that let shares the binding environment (recursive):
        // nix-repl> let a = b; b = 10; in a
        // 10
        // nix-repl> let b = 10; a = b; in a
        // 10
        // }
        (Expr::App { f, args, arity }, _) => {
            stack.write(mc).push(Cont::ApplyCont {
                env: env,
                args: args.to_vec(),
                arity: *arity,
            });
            (*f, env)
        }
        // (Expr::Thunk { t, env }, _) => {
        //     stack.push(Gc::allocate(mc, Cont::UpdateCont {
        //         update_expr: expr, // use gc pointer here, not value
        //     }));
        //     // need really only one blackhole here, can do pointer comparison.
        //     expr = black_hole;
        //     (f, env)
        // },
        (Expr::PrimOp { name, arity: op_arity }, Some(Cont::ApplyCont { arity: ap_arity, args, .. })) => {
            match op_arity.cmp(&ap_arity) {
                // apply `arity` arguments to primop, push new applycont with
                // remaining args
                Ordering::Less => {
                    unreachable!("execute primop and push new application cont")
                }
                Ordering::Equal => {
                    unreachable!("execute primop")
                },
                Ordering::Greater => {
                    let expr2 = Gc::allocate(mc, Expr::Pap {
                        f: expr,
                        args: args.to_vec(),
                        arity: op_arity - ap_arity,
                    });
                    (expr2, env)
                },
            }
        }
        (Expr::Lambda { .. }, _) => {
            // check top of stack, follow call rules
            (expr, env)
        }
        _default => {
            println!("ret default");
            (expr, env)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // #[test]
    // fn check_done() {
    //     let i = Expr::Int(10);
    //     let env = Environment::new();
    //     i.eval(&env);
    // }
    #[test]
    fn check_application() {
        let mut arena = ExprArena::new(ArenaParameters::default(), |mc| ExprRoot {
            // 2 + 1
            // ((+ 2) 1)
            root: Gc::allocate(
                mc,
                Expr::App {
                    f: Gc::allocate(
                        mc,
                        Expr::App {
                            f: Gc::allocate(mc, Expr::PrimOp { name: "plus", arity: 2 }),
                            arity: 1,
                            args: vec![Gc::allocate(mc, Expr::Int(2))],
                        },
                    ),
                    arity: 1,
                    args: vec![Gc::allocate(mc, Expr::Int(1))],
                },
            ),
            stack: GcCell::allocate(mc, Vec::new()),
            env: Gc::allocate(mc, Env::new_root()),
        });
        arena.mutate(|mc, root| {
            let black_hole = Gc::allocate(mc, Expr::Null());
            let (f, env) = (root.root, root.env);
            let (f, env) = step(mc, f, env, black_hole, root.stack);
            let (f, env) = step(mc, f, env, black_hole, root.stack);
            let (f, env) = step(mc, f, env, black_hole, root.stack);
            let (f, env) = step(mc, f, env, black_hole, root.stack);
            println!("eval: {:?}", f);
        });
    }
}
