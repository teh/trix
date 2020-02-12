#[allow(unused_imports)]
use crate::expr::{Cont, Env, Expr, ExprArena, ExprRoot, GcEnv, GcExpr, GcStack};
use gc_arena::{make_arena, ArenaParameters, Collect, Gc, GcCell, MutationContext};
use std::cmp::Ordering;
#[macro_use] extern crate lalrpop_util;


pub mod expr;
pub mod parser;
lalrpop_mod!(pub expr_parser);


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
                Expr::PrimOp {
                    arity: op_arity, name, ..
                } => {
                    match op_arity.cmp(&arity) {
                        // apply `arity` arguments to primop, push new applycont with
                        // remaining args
                        Ordering::Less => {
                            // rule CALLK
                            unreachable!("execute primop and push new application cont")
                        }
                        Ordering::Equal => {
                            // rule EXACT

                            // TODO what if args not evaluated yet? Probably
                            // need to push a ForceEvalCont and return the first
                            // arg to be evaluated. When that returns
                            // ForceEvalCont pops of one Arg, force evals the
                            // next one etc until done, then re-does the App.
                            let l = (*args)[0];
                            let r = (*args)[1];
                            match (name, &*l, &*r) {
                                ("plus", Expr::Int(left), Expr::Int(right)) => {
                                    let expr2 = Gc::allocate(mc, Expr::Int(left + right));
                                    (expr2, env)
                                }
                                _ => unreachable!("invalid op {}/{}", name, arity),
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
            stack.write(mc).pop();
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
        (Expr::Thunk { t }, _) => {
            // TODO - blackholing
            (*t, env)
        }
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
    use gc_arena::rootless_arena;

    #[test]
    fn check_pap_primop() {
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
            // TODO - need function that is essentialy `eval` that runs until no
            // redex left.
            for i in 0..10 {
                s = step(mc, s.0, s.1, black_hole, root.stack);
                match *s.0 {
                    Expr::Int(v) => {
                        assert_eq!(v, 3);
                        break;
                    }
                    _ => (),
                }
            }
        });
    }

    #[test]
    fn check_app_primop() {
        let mut arena = ExprArena::new(ArenaParameters::default(), |mc| ExprRoot {
            root: Gc::allocate(
                mc,
                Expr::App {
                    f: Gc::allocate(mc, Expr::PrimOp { name: "plus", arity: 2 }),
                    arity: 2,
                    args: vec![Gc::allocate(mc, Expr::Int(2)), Gc::allocate(mc, Expr::Int(1))],
                },
            ),
            stack: GcCell::allocate(mc, Vec::new()),
            env: Gc::allocate(mc, Env::new_root()),
        });
        arena.mutate(|mc, root| {
            let black_hole = Gc::allocate(mc, Expr::Null());
            let mut s = (root.root, root.env);
            // TODO - need function that is essentialy `eval` that runs until no
            // redex left.
            for i in 0..10 {
                s = step(mc, s.0, s.1, black_hole, root.stack);
                match *s.0 {
                    Expr::Int(v) => {
                        assert_eq!(v, 3);
                        break;
                    }
                    _ => (),
                }
            }
        });
    }

    #[test]
    fn check_thunk() {
        rootless_arena(|mc| {
            let root = ExprRoot {
                root: Gc::allocate(
                    mc,
                    Expr::Thunk {
                        t: Gc::allocate(mc, Expr::String("thunk".to_string())),
                    },
                ),
                stack: GcCell::allocate(mc, Vec::new()),
                env: Gc::allocate(mc, Env::new_root()),
            };
            let black_hole = Gc::allocate(mc, Expr::Null());
            let mut s = (root.root, root.env);
            for i in 0..10 {
                s = step(mc, s.0, s.1, black_hole, root.stack);
                match *(s.0) {
                    Expr::String(ref s) => {
                        assert_eq!(s, "thunk");
                        break;
                    }
                    _ => (),
                }
            }
        });
    }
}
