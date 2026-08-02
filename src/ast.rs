#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Builtin {
    Abs,
    Acos,
    Asin,
    Atan,
    Atan2,
    Ceil,
    Cos,
    Cosh,
    E,
    Exp,
    Fac,
    Floor,
    Ln,
    Log,
    Log10,
    Ncr,
    Npr,
    Pi,
    Pow,
    Sin,
    Sinh,
    Sqrt,
    Tan,
    Tanh,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),

    Variable(String),

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    Function {
        function: Builtin,
        args: Vec<Expr>,
    },

    /// The comma operator: `a, b, c`. Every element is evaluated in order
    /// (so side effects / errors in `a` and `b` still surface), but the
    /// value of the whole expression is the value of the last element.
    /// Always holds at least 2 elements — a single expression is never
    /// wrapped in a `Sequence`.
    Sequence(Vec<Expr>),
}