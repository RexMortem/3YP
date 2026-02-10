use std::fmt;

#[derive(Debug, Clone)]
pub enum Dist {
    Uniform(Box<Expr>, Box<Expr>), // start, end
    Discrete(Vec<(Box<Expr>, Box<Expr>)>), // value:probability pairs
    CombinedDist(Box<Dist>, Box<Dist>), // combining two distributions by summing outcomes
    ChainDist(Box<Dist>, u64, Box<Dist>), // current distribution, number of iterations, the distribution to join onto
    // BranchDist()
}

// #[derive(Debug, Clone)]
// pub enum Exp_Value<T> {
//     Exp_Value {
//         val: T,
       
//     }
// }

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
    HardcodedOutput(Expr)
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Var(String),
    // FuncCall(String, Vec<Expr>),

    // unary
    Neg(Box<Expr>),

    // binary
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    // distributions
    Dist(Dist),
    DistMethodCall {
        var: String,
        method: String,
        args: Vec<Expr>,
    },
}

// functionality for enums



// display opt-ins
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Float(fl) => write!(f, "{}", fl),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Add(lhs, rhs) => write!(f, "({} + {})", lhs, rhs),
            Expr::Sub(lhs, rhs) => write!(f, "({} - {})", lhs, rhs),
            Expr::Mul(lhs, rhs) => write!(f, "({} * {})", lhs, rhs),
            Expr::Div(lhs, rhs) => write!(f, "({} / {})", lhs, rhs),
            Expr::Neg(inner) => write!(f, "-{}", inner),
            Expr::Dist(dist) => write!(f, "{}", format_dist(dist)),
            Expr::DistMethodCall { var, method, args } => {
                write!(f, "{}:{}(", var, method)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

fn format_dist(dist: &Dist) -> String {
    match dist {
        Dist::Uniform(a, b) => format!("uniform({}, {})", a, b),
        Dist::Discrete(pairs) => {
            let pair_strs: Vec<String> = pairs.iter()
                .map(|(value, prob)| format!("{}:{}", value, prob))
                .collect();
            format!("Discrete({})", pair_strs.join(", "))
        }
        Dist::CombinedDist(d1, d2) => format!("({} + {})", format_dist(d1), format_dist(d2)),
        Dist::ChainDist(d1, n, d2) => format!("{}[{}]{}", format_dist(d1), n, format_dist(d2)),
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::DeclAssign { name, value } => write!(f, "let {} = {};", name, value),
            Statement::Decl(name) => write!(f, "let {};", name),
            Statement::HardcodedOutput(expr) => write!(f, "output({});", expr),
            Statement::Assign { name, value } => write!(f, "{} = {};", name, value),
        }
    }
}

