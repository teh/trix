use gc_arena::{make_arena, ArenaParameters, Collect, Gc, GcCell, MutationContext};
use std::collections::HashMap;

/// Placeholder for e.g. argument names in lambdas (x, y, i)
pub type Symbol = String;

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
///
pub enum Expr<'gc> {
    Null(),
    Int(i64),
    Float(f64),
    Bool(bool),
    Var(Symbol),

    // TODO should probably keep `Formal` in a separate structure
    Formal(Symbol, Option<GcExpr<'gc>>),
    InheritedVar(Symbol),
    // TODO: what about expressions in the string?
    String(String),
    Path(String),
    List(Vec<GcExpr<'gc>>),
    Attrs {
        attrs: HashMap<String, GcExpr<'gc>>,
        recursive: bool,
    },
    Assert {
        expr: GcExpr<'gc>,
        body: GcExpr<'gc>,
    },
    With {
        expr: GcExpr<'gc>,
        body: GcExpr<'gc>,
    },
    IfThenElse {
        if_expr: GcExpr<'gc>,
        then_expr: GcExpr<'gc>,
        else_expr: GcExpr<'gc>,
    },
    Lambda {
        arg: Option<Symbol>,
        body: GcExpr<'gc>,
        formals: Vec<GcExpr<'gc>>,
    },
    App {
        f: GcExpr<'gc>,
        args: Vec<GcExpr<'gc>>,
        arity: usize,
    },
    Select {
        expr: GcExpr<'gc>,
        attr_path: Vec<GcExpr<'gc>>,
    },
    SelectOr {
        expr: GcExpr<'gc>,
        attr_path_left: Vec<GcExpr<'gc>>,
        attr_path_right: Vec<GcExpr<'gc>>,
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
    BinaryOp { // specialized primop, might not need, to be decided later
        name: &'gc str,
        left: GcExpr<'gc>,
        right: GcExpr<'gc>,
    },
    UnaryMinus {
        expr: GcExpr<'gc>,
    },
    UnaryNot {
        expr: GcExpr<'gc>,
    },
    Let {
        bindings: HashMap<String, GcExpr<'gc>>,
        rec_bindings: HashMap<String, GcExpr<'gc>>, // TODO replace with either-like struct
        inherited: HashMap<String, GcExpr<'gc>>,    // inherit always inherits from parent env
        body: GcExpr<'gc>,                          // let ...; in body
    },
}

impl<'gc> PartialEq for Expr<'gc> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Int(s), Expr::Int(o)) => s == o,
            _ => unimplemented!("cannot eq-compare {:?} with {:?} yet", self, other),
        }
    }
}

impl<'gc> Eq for Expr<'gc> {}

impl<'gc> Expr<'gc> {}

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
    ForceEvalCont {
        args: Vec<GcExpr<'gc>>,
        arity: usize,
    },
}

impl<'gc> Env<'gc> {
    pub fn new_root() -> Env<'gc> {
        Env {
            up: None,
            values: HashMap::new(),
        }
    }
}

pub type GcExpr<'gc> = Gc<'gc, Expr<'gc>>;
pub type GcEnv<'gc> = Gc<'gc, Env<'gc>>;
pub type GcStack<'gc> = GcCell<'gc, Vec<Cont<'gc>>>;

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct ExprRoot<'gc> {
    pub root: GcExpr<'gc>,
    pub env: GcEnv<'gc>,
    pub stack: GcStack<'gc>,
}

make_arena!(pub ExprArena, ExprRoot);
