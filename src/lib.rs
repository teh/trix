use gc_arena::{make_arena, ArenaParameters, Collect, Gc, GcCell, MutationContext};
use std::cmp::Ordering;
use std::collections::HashMap;

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
    _black_hole: GcExpr<'gc>,
    stack: GcStack<'gc>,
) -> (GcExpr<'gc>, GcEnv<'gc>) {
    println!("\n");
    println!("step {:?}", *expr);
    println!("   s {:?}", stack.read());

    let stack_top = stack.read().last().cloned();
    match (&*expr, stack_top) {
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
        (Expr::App { f, args, arity, .. }, _) => {
            // TODO only push ApplyCont if either arity mismatch _or_ f is not
            // pointing to a Lambda or PrimOp yet
            println!("app {:?}", **f);
            match **f {
                Expr::PrimOp { arity: op_arity, name, .. } => {
                    match op_arity.cmp(&arity) {
                        // apply `arity` arguments to primop, push new applycont with
                        // remaining args
                        Ordering::Less => {
                            // rule CALLK
                            unreachable!("execute primop and push new application cont")
                        }
                        Ordering::Equal => {
                            // rule EXACT
                            if name == "plus" {
                                let expr2 = Gc::allocate(
                                    mc, Expr::Int(10)
                                );
                                (expr2, env)
                            } else {
                                unreachable!("execute primop {}", name)
                            }
                        }
                        Ordering::Greater => {
                            // rule PAP
                            let expr2 = Gc::allocate(
                                mc,
                                Expr::Pap {
                                    f: *f,
                                    args: args.to_vec(),
                                    arity: op_arity - arity,
                                },
                            );
                            (expr2, env)
                        }
                    }
                }
                ref _default => {
                    // rule TCALL
                    stack.write(mc).push(Cont::ApplyCont {
                        env: env,
                        args: args.to_vec(),
                        arity: *arity,
                    });
                    (*f, env)
                }
            }
        }
        (
            Expr::Pap { f, args, arity },
            Some(Cont::ApplyCont {
                args: ref cont_args,
                arity: cont_arity,
                ..
            }),
        ) => {
            // partial apply just mops up new arguments and returns a normal
            // apply.
            let mut newargs = args.to_vec();
            newargs.extend(cont_args.to_vec());
            let expr2 = Gc::allocate(
                mc,
                Expr::App {
                    f: *f,
                    args: newargs,
                    arity: arity + cont_arity,
                },
            );
            (expr2, env)
        }
        // (Expr::Thunk { t, env }, _) => {
        //     stack.push(Gc::allocate(mc, Cont::UpdateCont {
        //         update_expr: expr, // use gc pointer here, not value
        //     }));
        //     // need really only one blackhole here, can do pointer comparison.
        //     expr = black_hole;
        //     (f, env)
        // },
        // (
        //     Expr::PrimOp { name, arity: op_arity },
        //     Some(Cont::ApplyCont {
        //         arity: ap_arity, args, ..
        //     }),
        // ) => {
        //     stack.write(mc).pop(); // we're handling the ApplyCont, pop from stack

        // }
        // Expr::Lambda { .. } => {
        //     // check top of stack, follow call rules
        //     unreachable!()
        //     // (expr, env)
        // }
        _ => {
            unreachable!()
            // (expr, env)
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
            let mut s = (root.root, root.env);
            for _ in 0..5 {
                s = step(mc, s.0, s.1, black_hole, root.stack);
            }
            println!("eval: {:?}", s);
        });
    }
}
