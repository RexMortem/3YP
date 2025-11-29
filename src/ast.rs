use std::fmt;

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
    Var(String),

    // unary
    Neg(Box<Expr>),

    // binary
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

// functionality for enums



// display opt-ins
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Add(lhs, rhs) => write!(f, "({} + {})", lhs, rhs),
            Expr::Sub(lhs, rhs) => write!(f, "({} - {})", lhs, rhs),
            Expr::Mul(lhs, rhs) => write!(f, "({} * {})", lhs, rhs),
            Expr::Div(lhs, rhs) => write!(f, "({} / {})", lhs, rhs),
            Expr::Neg(inner) => write!(f, "-{}", inner),
        }
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