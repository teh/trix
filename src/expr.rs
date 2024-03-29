use gc_arena::{make_arena, ArenaParameters, Collect, Gc, GcCell};
use std::collections::HashMap;

/// Placeholder for e.g. argument names in lambdas (x, y, i)
pub type Symbol = String;

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
///
pub enum Expr<'gc> {
    Null(),
    Int(i64),

    // TODO decide how to handle floats
    Float(String),
    Bool(bool),
    Var(Symbol),

    // TODO should probably keep `Formal` in a separate structure
    Formal(Symbol, Option<GcExpr<'gc>>),
    InheritedVar(Symbol),
    String(String),

    // Interpolated strings are made up of expressions (either Expr::String or
    // some other expression that must evaluate to Expr::String)
    InterpolatedString(Vec<GcExpr<'gc>>),
    Path(String),
    List(Vec<GcExpr<'gc>>),
    Attrs {
        // unfortunately left-side attributes can be dynamic, e.g.
        // let xx = "xx"; in { ${xx} = 2; } is totally valid.
        attrs: Vec<(Vec<GcExpr<'gc>>, GcExpr<'gc>)>,
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
        // bool == has ellipsis ...
        formals: (Vec<GcExpr<'gc>>, bool),
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
    HasAttr { // specialized primop, might not need, to be decided later
        expr: GcExpr<'gc>,
        attr_path: Vec<GcExpr<'gc>>,
    },
    UnaryMinus {
        expr: GcExpr<'gc>,
    },
    UnaryNot {
        expr: GcExpr<'gc>,
    },
    Let {
        // let is the only place where dynamic attributes are disallowed
        bindings: Vec<(Vec<GcExpr<'gc>>, GcExpr<'gc>)>,
        // inherited: HashMap<String, GcExpr<'gc>>,    // inherit always inherits from parent env
        body: GcExpr<'gc>,                          // let ...; in body
    },
}

impl<'gc> PartialEq for Expr<'gc> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Int(s), Expr::Int(o)) => s == o,
            _ => unimplemented!("cannot eq-compare {:?} with {:?}", self, other),
        }
    }
}

impl<'gc> Eq for Expr<'gc> {}

impl<'gc> Expr<'gc> {
    /// Values cannot be evaluated any further. This matters when we force
    /// arguments e.g. for binary ops, but also for the actual evaluation.
    pub fn is_value(&self) -> bool {
        match self {
            Expr::Null() => true,
            Expr::Int(_) => true,
            Expr::Float(_) => true,
            Expr::Bool(_) => true,
            Expr::String(_) => true,
            Expr::InterpolatedString(_) => true,
            Expr::Path(_) => true,
            Expr::List(_) => true,
            Expr::Attrs  { .. } => true,
            Expr::Pap { .. } => true,
            _ => false,
        }
    }
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
    ForceAppCont {
        f: GcExpr<'gc>, /// lambda to apply to after all args have been forced
        unforced_args: Vec<GcExpr<'gc>>, // we consume unforced and append to forced args.
        forced_args: Vec<GcExpr<'gc>>,
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
