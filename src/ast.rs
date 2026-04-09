use std::fmt;

// Type System

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    /// A user-defined enum type, e.g. `Weather`. Must start with an uppercase letter.
    Named(String),
    /// A typed discrete distribution, e.g. `Discrete<Weather>`.
    DistOf(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Named(n) => write!(f, "{}", n),
            Type::DistOf(n) => write!(f, "Discrete<{}>", n),
        }
    }
}

// Enum Definitions

/// A user-defined enumeration type declaration.
/// e.g. `enum Weather { Sunny, Cloudy, Rainy }`
/// Type name and all variants must start with an uppercase letter.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
}

// Error Classification

/// Error class for probabilistic functions, determining how multiple rounds combine.
#[derive(Debug, Clone)]
pub enum ErrorClass {
    /// One-sided error: "no" is always correct, "yes" (Uncertain) may be wrong.
    /// Repeated rounds multiply error probabilities. Stop early on Certain(x).
    RP,
    /// One-sided error: "yes" is always correct, "no" (Uncertain) may be wrong.
    /// Mirror of RP. Stop early on Certain(x).
    CoRP,
    /// Two-sided error: both answers may be wrong with prob < 1/2.
    /// Use majority vote over k rounds; confidence from Chernoff bound.
    BPP,
}

impl fmt::Display for ErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorClass::RP => write!(f, "RP"),
            ErrorClass::CoRP => write!(f, "coRP"),
            ErrorClass::BPP => write!(f, "BPP"),
        }
    }
}

// Function Definitions

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub name: String,
    pub ty: Type,
}

/// A regular (deterministic) function definition.
#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

/// A probabilistic function definition with error metadata.
#[derive(Debug, Clone)]
pub struct PbFuncDef {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub return_type: Type,
    pub error_class: ErrorClass,
    /// Name of the distribution family describing error decay (e.g. "Geometric").
    pub error_distribution: String,
    pub body: Vec<Statement>,
}

// Top-Level Program Items

#[derive(Debug, Clone)]
pub enum ProgramItem {
    Statement(Statement),
    FuncDef(FuncDef),
    PbFuncDef(PbFuncDef),
    EnumDef(EnumDef),
}

// Distributions

#[derive(Debug, Clone)]
pub enum Dist {
    Uniform(Box<Expr>, Box<Expr>),           // start, end (discrete, inclusive)
    UniformContinuous(Box<Expr>, Box<Expr>),  // start, end (continuous)
    Discrete(Vec<(Box<Expr>, Box<Expr>)>),   // value:probability pairs
    CombinedDist(Box<Dist>, Box<Dist>),      // sum of two independent distributions
    Bernoulli(Box<Expr>),                    // p: probability of true
    Binomial(Box<Expr>, Box<Expr>),          // n: trials, p: success probability
    Geometric(Box<Expr>),                    // p: success probability per trial
    /// Bayesian Beta posterior: Beta(alpha, beta).
    /// Produced by `distribution_of(..., bayesian, N)`.
    Beta(Box<Expr>, Box<Expr>),              // alpha, beta
}

// Distribution-of Extraction Mode

/// How to extract the underlying distribution of a probabilistic function.
#[derive(Debug, Clone)]
pub enum DistributionOfMode {
    /// Use the error class metadata (RP → Geometric(0.5); BPP → Bernoulli(0.75)).
    Analytical,
    /// Run N single rounds; report empirical Certain probability as Bernoulli(p).
    Empirical(i64),
    /// Run N single rounds; produce a Beta posterior over the Certain probability.
    Bayesian(i64),
}

// Statements

#[derive(Debug, Clone)]
pub enum Statement {
    Decl(Expr),
    Assign {
        name: Expr,
        value: Expr,
    },
    DeclAssign {
        name: Expr,
        value: Expr,
    },
    HardcodedOutput(Expr),
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    /// `let result_var, info_var = func_name(args) with confidence >= confidence`
    PbCallAssign {
        result_var: String,
        info_var: String,
        func_name: String,
        args: Vec<Expr>,
        confidence: f64,
    },
    /// `let var = map(func_name, array_expr) [with confidence >= confidence]`
    /// When confidence is Some(c), func_name must be a pb function;
    /// the error budget is split evenly across all array elements (union bound).
    /// When confidence is None, func_name must be a regular function.
    MapCallAssign {
        var: String,
        func_name: String,
        array_expr: Expr,
        confidence: Option<f64>,
    },
    /// `let var = distribution_of(func_name(args), mode[, N])`
    /// Extracts the implicit underlying distribution of a pb function's per-round behaviour.
    DistributionOf {
        var: String,
        func_name: String,
        args: Vec<Expr>,
        mode: DistributionOfMode,
    },
}

// Expressions

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Var(String),

    // unary
    Neg(Box<Expr>),
    Not(Box<Expr>),

    // arithmetic
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),

    // comparison
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),

    // logical
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),

    // distributions: method call on a named distribution variable (legacy `:` syntax)
    DistMethodCall {
        var: String,
        method: String,
        args: Vec<Expr>,
    },

    // method call on any expression value (`.` syntax, e.g. `uniform(1,6).sample()`)
    ExprMethodCall {
        expr: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    // array literal
    Array(Vec<Expr>),

    // distribution literal
    Dist(Dist),

    // probabilistic certainty markers (returned from pb functions)
    Certain(Box<Expr>),
    Uncertain(Box<Expr>),

    // function call
    FuncCall(String, Vec<Expr>),

    // approximate equality: lhs ~= rhs (within tolerance)?
    // Default tolerance is 0.05.
    ApproxEq(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
}

// Display Helpers

pub fn format_dist(dist: &Dist) -> String {
    match dist {
        Dist::Uniform(a, b) => format!("uniform({}, {})", a, b),
        Dist::UniformContinuous(a, b) => format!("uniformContinuous({}, {})", a, b),
        Dist::Discrete(pairs) => {
            let pair_strs: Vec<String> = pairs
                .iter()
                .map(|(v, p)| format!("{}:{}", v, p))
                .collect();
            format!("Discrete({})", pair_strs.join(", "))
        }
        Dist::CombinedDist(d1, d2) => format!("({} + {})", format_dist(d1), format_dist(d2)),
        Dist::Bernoulli(p) => format!("Bernoulli({})", p),
        Dist::Binomial(n, p) => format!("Binomial({}, {})", n, p),
        Dist::Geometric(p) => format!("Geometric({})", p),
        Dist::Beta(alpha, beta) => format!("Beta({}, {})", alpha, beta),
    }
}

fn fmt_args(args: &[Expr]) -> String {
    args.iter()
        .map(|a| format!("{}", a))
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Float(fl) => write!(f, "{}", fl),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Neg(inner) => write!(f, "(-{})", inner),
            Expr::Not(inner) => write!(f, "(!{})", inner),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Sub(l, r) => write!(f, "({} - {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
            Expr::Div(l, r) => write!(f, "({} / {})", l, r),
            Expr::Mod(l, r) => write!(f, "({} % {})", l, r),
            Expr::Eq(l, r) => write!(f, "({} == {})", l, r),
            Expr::Neq(l, r) => write!(f, "({} != {})", l, r),
            Expr::Lt(l, r) => write!(f, "({} < {})", l, r),
            Expr::Lte(l, r) => write!(f, "({} <= {})", l, r),
            Expr::Gt(l, r) => write!(f, "({} > {})", l, r),
            Expr::Gte(l, r) => write!(f, "({} >= {})", l, r),
            Expr::And(l, r) => write!(f, "({} && {})", l, r),
            Expr::Or(l, r) => write!(f, "({} || {})", l, r),
            Expr::Array(elems) => {
                let parts: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            Expr::Dist(d) => write!(f, "{}", format_dist(d)),
            Expr::DistMethodCall { var, method, args } => {
                write!(f, "{}:{}({})", var, method, fmt_args(args))
            }
            Expr::ExprMethodCall { expr, method, args } => {
                write!(f, "{}.{}({})", expr, method, fmt_args(args))
            }
            Expr::Certain(inner) => write!(f, "Certain({})", inner),
            Expr::Uncertain(inner) => write!(f, "Uncertain({})", inner),
            Expr::FuncCall(name, args) => write!(f, "{}({})", name, fmt_args(args)),
            Expr::ApproxEq(l, r, None) => write!(f, "({} ~= {})", l, r),
            Expr::ApproxEq(l, r, Some(t)) => write!(f, "({} ~= {} within {})", l, r, t),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Decl(name) => write!(f, "let {};", name),
            Statement::DeclAssign { name, value } => write!(f, "let {} = {};", name, value),
            Statement::Assign { name, value } => write!(f, "{} = {};", name, value),
            Statement::HardcodedOutput(expr) => write!(f, "output({});", expr),
            Statement::Return(Some(expr)) => write!(f, "return {};", expr),
            Statement::Return(None) => write!(f, "return;"),
            Statement::If { cond, then_block, else_block } => {
                write!(f, "if {} {{ {} statements }}", cond, then_block.len())?;
                if let Some(else_b) = else_block {
                    write!(f, " else {{ {} statements }}", else_b.len())?;
                }
                Ok(())
            }
            Statement::PbCallAssign { result_var, info_var, func_name, args, confidence } => {
                write!(
                    f,
                    "let {}, {} = {}({}) with confidence >= {};",
                    result_var, info_var, func_name, fmt_args(args), confidence
                )
            }
            Statement::MapCallAssign { var, func_name, array_expr, confidence } => {
                match confidence {
                    Some(c) => write!(
                        f,
                        "let {} = map({}, {}) with confidence >= {};",
                        var, func_name, array_expr, c
                    ),
                    None => write!(f, "let {} = map({}, {});", var, func_name, array_expr),
                }
            }
            Statement::DistributionOf { var, func_name, args, mode } => {
                let mode_str = match mode {
                    DistributionOfMode::Analytical => "analytical".to_string(),
                    DistributionOfMode::Empirical(n) => format!("empirical, {}", n),
                    DistributionOfMode::Bayesian(n) => format!("bayesian, {}", n),
                };
                write!(f, "let {} = distribution_of({}({}), {});", var, func_name, fmt_args(args), mode_str)
            }
        }
    }
}
