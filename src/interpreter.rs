use std::collections::HashMap;
use std::fmt;

use fraction::Fraction;
use fraction::ToPrimitive;
use rand::Rng;

use crate::ast::*;
use crate::visualiser::{self, HistogramData, HistKind};

// ── Runtime Value ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Dist(Dist),
    /// Returned by a pb function round to signal a definitive result.
    Certain(Box<RuntimeValue>),
    /// Returned by a pb function round to signal a probabilistic result.
    Uncertain(Box<RuntimeValue>),
    /// Metadata produced alongside a pb function call result.
    Info { rounds: u64, confidence: f64 },
    /// An ordered collection of runtime values.
    Array(Vec<RuntimeValue>),
    /// A distribution histogram ready to be rendered (produced by `:visualise()`).
    Visualisation(HistogramData),
    /// An exact rational probability value, produced by analytical distribution queries
    /// such as `:expect()`, `:mean()` on discrete distributions.
    Frac(Fraction),
    /// A value of a user-defined enum type, e.g. `Sunny` of type `Weather`.
    EnumVariant(String, String), // (type_name, variant_name)
    /// A dynamic discrete distribution over arbitrary state values.
    /// Produced by `bind()` and `step()` for Markov chain computations.
    DynDist(Vec<(RuntimeValue, f64)>),
}

// ── Output Line ───────────────────────────────────────────────────────────────

/// A single item in the program's output stream.
/// Text lines are printed as-is; histograms are rendered differently for CLI
/// (ASCII) vs web (SVG).
pub enum OutputLine {
    Text(String),
    Hist(HistogramData),
}

impl RuntimeValue {
    /// Extract a numeric value (Int, Float, or Frac) as f64, panicking otherwise.
    fn as_f64(&self) -> f64 {
        match self {
            RuntimeValue::Int(n) => *n as f64,
            RuntimeValue::Float(n) => *n,
            RuntimeValue::Frac(f) => f.to_f64().unwrap_or(f64::NAN),
            other => panic!("Type error: expected a number, got {}", other),
        }
    }

    /// Extract a boolean, panicking otherwise.
    fn as_bool(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            other => panic!("Type error: expected bool, got {}", other),
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Int(n) => write!(f, "{}", n),
            RuntimeValue::Float(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            RuntimeValue::Bool(b) => write!(f, "{}", b),
            RuntimeValue::Dist(d) => write!(f, "{}", format_dist(d)),
            RuntimeValue::Certain(inner) => write!(f, "Certain({})", inner),
            RuntimeValue::Uncertain(inner) => write!(f, "Uncertain({})", inner),
            RuntimeValue::Info { rounds, confidence } => {
                write!(f, "Info {{ rounds: {}, confidence: {:.6} }}", rounds, confidence)
            }
            RuntimeValue::Array(elems) => {
                let parts: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", parts.join(", "))
            }
            RuntimeValue::Visualisation(data) => write!(f, "<Visualisation: {}>", data.label),
            RuntimeValue::Frac(frac) => write!(f, "{}", frac),
            RuntimeValue::EnumVariant(_, variant) => write!(f, "{}", variant),
            RuntimeValue::DynDist(outcomes) => {
                let parts: Vec<String> = outcomes
                    .iter()
                    .map(|(v, p)| format!("{}: {:.4}", v, p))
                    .collect();
                write!(f, "DynDist{{{}}}", parts.join(", "))
            }
        }
    }
}

// ── Flow Control ──────────────────────────────────────────────────────────────

enum FlowControl {
    Continue,
    Return(RuntimeValue),
}

// ── Runtime Environment ───────────────────────────────────────────────────────

struct RuntimeEnv {
    /// All runtime values (scalars, booleans, distributions) keyed by name.
    vars: HashMap<String, RuntimeValue>,
    /// Registered regular functions.
    funcs: HashMap<String, FuncDef>,
    /// Registered probabilistic functions.
    pb_funcs: HashMap<String, PbFuncDef>,
    /// User-defined enum type definitions: type_name → list of variant names.
    enum_types: HashMap<String, Vec<String>>,
    /// Reverse index: variant_name → type_name (for fast lookup at runtime).
    enum_variants: HashMap<String, String>,
    /// Lines accumulated by `output(...)` statements.
    output: Vec<OutputLine>,
}

impl RuntimeEnv {
    fn new() -> Self {
        RuntimeEnv {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            pb_funcs: HashMap::new(),
            enum_types: HashMap::new(),
            enum_variants: HashMap::new(),
            output: Vec::<OutputLine>::new(),
        }
    }

    /// Register all variants of an enum definition into both lookup maps.
    fn register_enum(&mut self, def: &EnumDef) {
        for variant in &def.variants {
            self.enum_variants.insert(variant.clone(), def.name.clone());
        }
        self.enum_types.insert(def.name.clone(), def.variants.clone());
    }

    /// Create a child environment for function calls, inheriting the function
    /// registries but starting with an empty variable scope and output buffer.
    fn new_child(&self) -> Self {
        RuntimeEnv {
            vars: HashMap::new(),
            funcs: self.funcs.clone(),
            pb_funcs: self.pb_funcs.clone(),
            enum_types: self.enum_types.clone(),
            enum_variants: self.enum_variants.clone(),
            output: Vec::<OutputLine>::new(),
        }
    }

    // ── Fraction Helpers ──────────────────────────────────────────────────────

    /// Convert a user-specified `f64` (parsed from source like `0.5`, `0.1`) to an
    /// exact `Fraction`.  Uses the shortest decimal string representation so that
    /// `0.1` → `1/10`, `0.5` → `1/2`, `0.75` → `3/4`, etc.
    fn float_to_frac(v: f64) -> Fraction {
        if v == 0.0 { return Fraction::from(0u64); }
        if v == 1.0 { return Fraction::from(1u64); }
        let s = format!("{}", v);
        if let Some(dot_pos) = s.find('.') {
            let frac_digits = &s[dot_pos + 1..];
            let decimal_places = frac_digits.len() as u32;
            let denom = 10u64.pow(decimal_places);
            // Integer numerator: concatenate integer and fractional digit strings
            let int_part = &s[..dot_pos];
            let combined = format!("{}{}", int_part.trim_start_matches('-'), frac_digits);
            let num: u64 = combined.parse().unwrap_or(0);
            let frac = Fraction::new(num, denom);
            if v < 0.0 { -frac } else { frac }
        } else {
            let n: u64 = s.parse().unwrap_or(0);
            Fraction::from(n)
        }
    }

    /// Convert an `i64` integer to a `Fraction`.
    fn int_to_frac(n: i64) -> Fraction {
        if n >= 0 {
            Fraction::from(n as u64)
        } else {
            -Fraction::from((-n) as u64)
        }
    }

    /// Round `v` to `sig` significant figures.
    /// Used to strip floating-point noise before converting to a Fraction.
    fn round_sig(v: f64, sig: u32) -> f64 {
        if v == 0.0 { return 0.0; }
        let magnitude = v.abs().log10().floor();
        let factor = 10f64.powi(sig as i32 - 1 - magnitude as i32);
        (v * factor).round() / factor
    }

    /// Raise `base` to the power `exp` using repeated multiplication.
    fn pow_frac(base: Fraction, exp: u64) -> Fraction {
        let mut result = Fraction::from(1u64);
        for _ in 0..exp {
            result = result * base.clone();
        }
        result
    }

    // ── Expression Evaluation ─────────────────────────────────────────────────

    fn eval_expr(&self, expr: &Expr) -> RuntimeValue {
        match expr {
            Expr::Int(n) => RuntimeValue::Int(*n),
            Expr::Float(n) => RuntimeValue::Float(*n),
            Expr::Bool(b) => RuntimeValue::Bool(*b),

            Expr::Var(name) => self
                .vars
                .get(name)
                .cloned()
                .unwrap_or_else(|| {
                    // Check if it's a known enum variant (e.g. Sunny, Cloudy).
                    if let Some(type_name) = self.enum_variants.get(name) {
                        RuntimeValue::EnumVariant(type_name.clone(), name.clone())
                    } else {
                        panic!("Undefined variable: '{}'", name)
                    }
                }),

            // ── Unary ─────────────────────────────────────────────────────────
            Expr::Neg(inner) => match self.eval_expr(inner) {
                RuntimeValue::Int(n) => RuntimeValue::Int(-n),
                RuntimeValue::Float(n) => RuntimeValue::Float(-n),
                v => panic!("Type error: cannot negate {}", v),
            },
            Expr::Not(inner) => match self.eval_expr(inner) {
                RuntimeValue::Bool(b) => RuntimeValue::Bool(!b),
                v => panic!("Type error: '!' requires bool, got {}", v),
            },

            // ── Arithmetic ────────────────────────────────────────────────────
            Expr::Add(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => RuntimeValue::Int(x + y),
                (RuntimeValue::Float(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x + y),
                (RuntimeValue::Int(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x as f64 + y),
                (RuntimeValue::Float(x), RuntimeValue::Int(y)) => RuntimeValue::Float(x + y as f64),
                // Exact fraction arithmetic
                (RuntimeValue::Frac(x), RuntimeValue::Frac(y)) => RuntimeValue::Frac(x + y),
                (RuntimeValue::Frac(x), RuntimeValue::Int(y))  => RuntimeValue::Frac(x + Self::int_to_frac(y)),
                (RuntimeValue::Int(x),  RuntimeValue::Frac(y)) => RuntimeValue::Frac(Self::int_to_frac(x) + y),
                // Frac + Float demotes to Float
                (RuntimeValue::Frac(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x.to_f64().unwrap_or(0.0) + y),
                (RuntimeValue::Float(x), RuntimeValue::Frac(y)) => RuntimeValue::Float(x + y.to_f64().unwrap_or(0.0)),
                // Combining two distributions analytically (sum of outcomes)
                (RuntimeValue::Dist(d1), RuntimeValue::Dist(d2)) => {
                    RuntimeValue::Dist(Dist::CombinedDist(Box::new(d1), Box::new(d2)))
                }
                (a, b) => panic!("Type error: cannot add {} and {}", a, b),
            },
            Expr::Sub(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => RuntimeValue::Int(x - y),
                (RuntimeValue::Float(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x - y),
                (RuntimeValue::Int(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x as f64 - y),
                (RuntimeValue::Float(x), RuntimeValue::Int(y)) => RuntimeValue::Float(x - y as f64),
                (RuntimeValue::Frac(x), RuntimeValue::Frac(y)) => RuntimeValue::Frac(x - y),
                (RuntimeValue::Frac(x), RuntimeValue::Int(y))  => RuntimeValue::Frac(x - Self::int_to_frac(y)),
                (RuntimeValue::Int(x),  RuntimeValue::Frac(y)) => RuntimeValue::Frac(Self::int_to_frac(x) - y),
                (RuntimeValue::Frac(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x.to_f64().unwrap_or(0.0) - y),
                (RuntimeValue::Float(x), RuntimeValue::Frac(y)) => RuntimeValue::Float(x - y.to_f64().unwrap_or(0.0)),
                (a, b) => panic!("Type error: cannot subtract {} from {}", b, a),
            },
            Expr::Mul(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => RuntimeValue::Int(x * y),
                (RuntimeValue::Float(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x * y),
                (RuntimeValue::Int(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x as f64 * y),
                (RuntimeValue::Float(x), RuntimeValue::Int(y)) => RuntimeValue::Float(x * y as f64),
                (RuntimeValue::Frac(x), RuntimeValue::Frac(y)) => RuntimeValue::Frac(x * y),
                (RuntimeValue::Frac(x), RuntimeValue::Int(y))  => RuntimeValue::Frac(x * Self::int_to_frac(y)),
                (RuntimeValue::Int(x),  RuntimeValue::Frac(y)) => RuntimeValue::Frac(Self::int_to_frac(x) * y),
                (RuntimeValue::Frac(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x.to_f64().unwrap_or(0.0) * y),
                (RuntimeValue::Float(x), RuntimeValue::Frac(y)) => RuntimeValue::Float(x * y.to_f64().unwrap_or(0.0)),
                (a, b) => panic!("Type error: cannot multiply {} and {}", a, b),
            },
            Expr::Div(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => {
                    if y == 0 { panic!("Runtime error: division by zero"); }
                    RuntimeValue::Float(x as f64 / y as f64)
                }
                (RuntimeValue::Float(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x / y),
                (RuntimeValue::Int(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x as f64 / y),
                (RuntimeValue::Float(x), RuntimeValue::Int(y)) => RuntimeValue::Float(x / y as f64),
                (RuntimeValue::Frac(x), RuntimeValue::Frac(y)) => RuntimeValue::Frac(x / y),
                (RuntimeValue::Frac(x), RuntimeValue::Int(y))  => RuntimeValue::Frac(x / Self::int_to_frac(y)),
                (RuntimeValue::Int(x),  RuntimeValue::Frac(y)) => RuntimeValue::Frac(Self::int_to_frac(x) / y),
                (a, b) => panic!("Type error: cannot divide {} by {}", a, b),
            },
            Expr::Mod(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Int(x), RuntimeValue::Int(y)) => RuntimeValue::Int(x.rem_euclid(y)),
                (RuntimeValue::Float(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x % y),
                (RuntimeValue::Int(x), RuntimeValue::Float(y)) => RuntimeValue::Float(x as f64 % y),
                (RuntimeValue::Float(x), RuntimeValue::Int(y)) => RuntimeValue::Float(x % y as f64),
                (a, b) => panic!("Type error: cannot compute {} mod {}", a, b),
            },

            // ── Comparison ────────────────────────────────────────────────────
            Expr::Eq(a, b) => RuntimeValue::Bool(self.eval_numeric_eq(a, b)),
            Expr::Neq(a, b) => RuntimeValue::Bool(!self.eval_numeric_eq(a, b)),
            Expr::Lt(a, b) => RuntimeValue::Bool(self.eval_expr(a).as_f64() < self.eval_expr(b).as_f64()),
            Expr::Lte(a, b) => RuntimeValue::Bool(self.eval_expr(a).as_f64() <= self.eval_expr(b).as_f64()),
            Expr::Gt(a, b) => RuntimeValue::Bool(self.eval_expr(a).as_f64() > self.eval_expr(b).as_f64()),
            Expr::Gte(a, b) => RuntimeValue::Bool(self.eval_expr(a).as_f64() >= self.eval_expr(b).as_f64()),

            // ── Logical ───────────────────────────────────────────────────────
            Expr::And(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Bool(x), RuntimeValue::Bool(y)) => RuntimeValue::Bool(x && y),
                (a, b) => panic!("Type error: '&&' requires bool operands, got {} and {}", a, b),
            },
            Expr::Or(a, b) => match (self.eval_expr(a), self.eval_expr(b)) {
                (RuntimeValue::Bool(x), RuntimeValue::Bool(y)) => RuntimeValue::Bool(x || y),
                (a, b) => panic!("Type error: '||' requires bool operands, got {} and {}", a, b),
            },

            // ── Arrays ────────────────────────────────────────────────────────
            Expr::Array(elems) => {
                RuntimeValue::Array(elems.iter().map(|e| self.eval_expr(e)).collect())
            }

            // ── Distributions ─────────────────────────────────────────────────
            Expr::Dist(d) => RuntimeValue::Dist(d.clone()),

            Expr::DistMethodCall { var, method, args } => {
                let dist = match self.vars.get(var) {
                    Some(RuntimeValue::Dist(d)) => d.clone(),
                    Some(v) => panic!("Type error: '{}' is not a distribution (got {})", var, v),
                    None => panic!("Undefined variable: {}", var),
                };
                self.eval_dist_method(&dist, method, args)
            }

            Expr::ExprMethodCall { expr, method, args } => {
                match self.eval_expr(expr) {
                    RuntimeValue::Dist(dist) => self.eval_dist_method(&dist, method, args),
                    RuntimeValue::DynDist(outcomes) => {
                        self.eval_dyn_dist_method(outcomes, method, args)
                    }
                    v => panic!("Type error: cannot call method '{}' on {}", method, v),
                }
            }

            // ── Certainty markers ─────────────────────────────────────────────
            Expr::Certain(inner) => RuntimeValue::Certain(Box::new(self.eval_expr(inner))),
            Expr::Uncertain(inner) => RuntimeValue::Uncertain(Box::new(self.eval_expr(inner))),

            // ── Function calls ────────────────────────────────────────────────
            Expr::FuncCall(name, args) => self.eval_func_call(name, args),

            // ── Approximate equality ──────────────────────────────────────────
            Expr::ApproxEq(a, b, tol_expr) => {
                let tolerance = tol_expr
                    .as_ref()
                    .map(|e| self.eval_expr(e).as_f64())
                    .unwrap_or(0.05);
                let result = match (self.eval_expr(a), self.eval_expr(b)) {
                    (RuntimeValue::Dist(d1), RuntimeValue::Dist(d2)) => {
                        self.dist_approx_eq(&d1, &d2, tolerance)
                    }
                    (lhs, rhs) => (lhs.as_f64() - rhs.as_f64()).abs() <= tolerance,
                };
                RuntimeValue::Bool(result)
            }
        }
    }

    fn eval_numeric_eq(&self, a: &Expr, b: &Expr) -> bool {
        match (self.eval_expr(a), self.eval_expr(b)) {
            (RuntimeValue::Bool(x), RuntimeValue::Bool(y)) => x == y,
            (RuntimeValue::Dist(d1), RuntimeValue::Dist(d2)) => self.dist_exact_eq(&d1, &d2),
            (RuntimeValue::EnumVariant(t1, v1), RuntimeValue::EnumVariant(t2, v2)) => {
                t1 == t2 && v1 == v2
            }
            (lhs, rhs) => lhs.as_f64() == rhs.as_f64(),
        }
    }

    // ── Distribution Equality ─────────────────────────────────────────────────

    /// Exact structural equality: same distribution type and identical evaluated parameters.
    fn dist_exact_eq(&self, d1: &Dist, d2: &Dist) -> bool {
        match (d1, d2) {
            (Dist::Uniform(a1, b1), Dist::Uniform(a2, b2)) => {
                self.eval_expr(a1).as_f64() == self.eval_expr(a2).as_f64()
                    && self.eval_expr(b1).as_f64() == self.eval_expr(b2).as_f64()
            }
            (Dist::UniformContinuous(a1, b1), Dist::UniformContinuous(a2, b2)) => {
                self.eval_expr(a1).as_f64() == self.eval_expr(a2).as_f64()
                    && self.eval_expr(b1).as_f64() == self.eval_expr(b2).as_f64()
            }
            (Dist::Bernoulli(p1), Dist::Bernoulli(p2)) => {
                self.eval_expr(p1).as_f64() == self.eval_expr(p2).as_f64()
            }
            (Dist::Binomial(n1, p1), Dist::Binomial(n2, p2)) => {
                self.eval_expr(n1).as_f64() == self.eval_expr(n2).as_f64()
                    && self.eval_expr(p1).as_f64() == self.eval_expr(p2).as_f64()
            }
            (Dist::Geometric(p1), Dist::Geometric(p2)) => {
                self.eval_expr(p1).as_f64() == self.eval_expr(p2).as_f64()
            }
            (Dist::Beta(a1, b1), Dist::Beta(a2, b2)) => {
                self.eval_expr(a1).as_f64() == self.eval_expr(a2).as_f64()
                    && self.eval_expr(b1).as_f64() == self.eval_expr(b2).as_f64()
            }
            (Dist::Discrete(pairs1), Dist::Discrete(pairs2)) => {
                if pairs1.len() != pairs2.len() {
                    return false;
                }
                let mut map1: HashMap<i64, f64> = HashMap::new();
                for (v, p) in pairs1 {
                    map1.insert(self.eval_expr(v).as_f64() as i64, self.eval_expr(p).as_f64());
                }
                for (v, p) in pairs2 {
                    let key = self.eval_expr(v).as_f64() as i64;
                    match map1.get(&key) {
                        Some(&p1) if (p1 - self.eval_expr(p).as_f64()).abs() < 1e-12 => {}
                        _ => return false,
                    }
                }
                true
            }
            (Dist::CombinedDist(d1a, d1b), Dist::CombinedDist(d2a, d2b)) => {
                self.dist_exact_eq(d1a, d2a) && self.dist_exact_eq(d1b, d2b)
            }
            _ => false,
        }
    }

    /// Returns true if the distribution is discrete (can be enumerated as integer outcomes).
    fn dist_is_discrete(&self, dist: &Dist) -> bool {
        match dist {
            Dist::Uniform(_,_) | Dist::Discrete(_) | Dist::Bernoulli(_)
            | Dist::Binomial(_,_) | Dist::Geometric(_) => true,
            Dist::CombinedDist(d1, d2) => self.dist_is_discrete(d1) && self.dist_is_discrete(d2),
            Dist::UniformContinuous(_,_) | Dist::Beta(_,_) => false,
            Dist::ChainDist(_,_,_) => false,
        }
    }

    /// Analytical mean of a distribution as f64.
    fn dist_mean_f64(&self, dist: &Dist) -> f64 {
        match dist {
            Dist::Uniform(a, b) | Dist::UniformContinuous(a, b) => {
                (self.eval_expr(a).as_f64() + self.eval_expr(b).as_f64()) / 2.0
            }
            Dist::Bernoulli(p) => self.eval_expr(p).as_f64(),
            Dist::Binomial(n, p) => self.eval_expr(n).as_f64() * self.eval_expr(p).as_f64(),
            Dist::Geometric(p) => 1.0 / self.eval_expr(p).as_f64(),
            Dist::Beta(alpha, beta) => {
                let a = self.eval_expr(alpha).as_f64();
                let b = self.eval_expr(beta).as_f64();
                a / (a + b)
            }
            Dist::Discrete(_) => {
                let outcomes = self.get_dist_outcomes(dist);
                outcomes.iter()
                    .map(|(v, p)| *v as f64 * p.to_f64().unwrap_or(0.0))
                    .sum()
            }
            Dist::CombinedDist(d1, d2) => self.dist_mean_f64(d1) + self.dist_mean_f64(d2),
            _ => panic!("dist_mean_f64: unsupported distribution type"),
        }
    }

    /// Analytical variance of a distribution as f64.
    fn dist_variance_f64(&self, dist: &Dist) -> f64 {
        match dist {
            Dist::Uniform(a, b) => {
                let n = self.eval_expr(b).as_f64() - self.eval_expr(a).as_f64() + 1.0;
                (n * n - 1.0) / 12.0
            }
            Dist::UniformContinuous(a, b) => {
                let range = self.eval_expr(b).as_f64() - self.eval_expr(a).as_f64();
                range * range / 12.0
            }
            Dist::Bernoulli(p) => {
                let pv = self.eval_expr(p).as_f64();
                pv * (1.0 - pv)
            }
            Dist::Binomial(n, p) => {
                let nv = self.eval_expr(n).as_f64();
                let pv = self.eval_expr(p).as_f64();
                nv * pv * (1.0 - pv)
            }
            Dist::Geometric(p) => {
                let pv = self.eval_expr(p).as_f64();
                (1.0 - pv) / (pv * pv)
            }
            Dist::Beta(alpha, beta) => {
                let a = self.eval_expr(alpha).as_f64();
                let b = self.eval_expr(beta).as_f64();
                let s = a + b;
                a * b / (s * s * (s + 1.0))
            }
            Dist::Discrete(_) => {
                let mean = self.dist_mean_f64(dist);
                let outcomes = self.get_dist_outcomes(dist);
                outcomes.iter()
                    .map(|(v, p)| {
                        let d = *v as f64 - mean;
                        d * d * p.to_f64().unwrap_or(0.0)
                    })
                    .sum()
            }
            Dist::CombinedDist(d1, d2) => {
                self.dist_variance_f64(d1) + self.dist_variance_f64(d2)
            }
            _ => panic!("dist_variance_f64: unsupported distribution type"),
        }
    }

    /// Approximate equality using Total Variation distance (discrete) or moment comparison (continuous).
    fn dist_approx_eq(&self, d1: &Dist, d2: &Dist, tolerance: f64) -> bool {
        let disc1 = self.dist_is_discrete(d1);
        let disc2 = self.dist_is_discrete(d2);

        if disc1 && disc2 {
            // Total Variation distance: 0.5 * Σ|p1(x) - p2(x)|
            let outcomes1 = self.get_dist_outcomes(d1);
            let outcomes2 = self.get_dist_outcomes(d2);

            let mut map1: HashMap<i64, f64> = HashMap::new();
            for (v, p) in &outcomes1 {
                *map1.entry(*v).or_insert(0.0) += p.to_f64().unwrap_or(0.0);
            }
            let mut map2: HashMap<i64, f64> = HashMap::new();
            for (v, p) in &outcomes2 {
                *map2.entry(*v).or_insert(0.0) += p.to_f64().unwrap_or(0.0);
            }

            // Union of all keys
            let mut all_keys: Vec<i64> = map1.keys().chain(map2.keys()).cloned().collect();
            all_keys.sort_unstable();
            all_keys.dedup();

            let tv: f64 = all_keys.iter()
                .map(|k| {
                    let p1 = map1.get(k).cloned().unwrap_or(0.0);
                    let p2 = map2.get(k).cloned().unwrap_or(0.0);
                    (p1 - p2).abs()
                })
                .sum::<f64>() * 0.5;

            tv <= tolerance
        } else if !disc1 && !disc2 {
            // CLT-inspired moment comparison: compare mean and std.
            // Both must be within `tolerance` (as a relative fraction of the larger std).
            let mean1 = self.dist_mean_f64(d1);
            let mean2 = self.dist_mean_f64(d2);
            let std1 = self.dist_variance_f64(d1).sqrt();
            let std2 = self.dist_variance_f64(d2).sqrt();

            let scale = std1.max(std2).max(1e-10);
            (mean1 - mean2).abs() / scale <= tolerance
                && (std1 - std2).abs() / scale <= tolerance
        } else {
            panic!("Cannot use ~= between a discrete and a continuous distribution")
        }
    }

    // ── Markov Chain / Dynamic Distribution Helpers ───────────────────────────

    /// A stable key for merging outcomes in bind/step.
    /// Uses "TypeName::Variant" for enum variants to avoid cross-enum collisions.
    fn dyn_key(v: &RuntimeValue) -> String {
        match v {
            RuntimeValue::EnumVariant(t, var) => format!("{}::{}", t, var),
            other => format!("{}", other),
        }
    }

    /// Enumerate (state, probability) pairs for any distribution as general `RuntimeValue`s.
    /// For enum-keyed `Discrete` distributions this evaluates each key expression and returns
    /// `RuntimeValue::EnumVariant`; for numeric distributions it wraps values as `RuntimeValue::Int`.
    fn get_dist_outcomes_dynamic(&self, dist: &Dist) -> Vec<(RuntimeValue, f64)> {
        match dist {
            // General Discrete: keys can be enum variants or numbers.
            Dist::Discrete(pairs) => pairs
                .iter()
                .map(|(v, p)| {
                    let val = self.eval_expr(v);
                    let prob = self.eval_expr(p).as_f64();
                    (val, prob)
                })
                .collect(),
            // All other distributions produce integer outcomes.
            _ => self
                .get_dist_outcomes(dist)
                .into_iter()
                .map(|(v, p)| (RuntimeValue::Int(v), p.to_f64().unwrap_or(0.0)))
                .collect(),
        }
    }

    /// Monadic bind for probability distributions (the Kleisli composition step).
    /// Given a distribution over states and a transition function S → Dist<S>,
    /// computes the resulting marginal distribution over new states by:
    ///   result(s') = Σ_s  P(s) · P_func(s)(s')
    fn eval_bind(&self, dist_val: RuntimeValue, func_name: &str) -> RuntimeValue {
        let outcomes: Vec<(RuntimeValue, f64)> = match &dist_val {
            RuntimeValue::Dist(d) => self.get_dist_outcomes_dynamic(d),
            RuntimeValue::DynDist(o) => o.clone(),
            v => panic!("bind() first argument must be a distribution, got {}", v),
        };

        let func = self
            .funcs
            .get(func_name)
            .cloned()
            .unwrap_or_else(|| panic!("bind(): undefined function '{}'", func_name));

        // Merge new outcomes: key → (RuntimeValue, accumulated_prob)
        let mut merged: HashMap<String, (RuntimeValue, f64)> = HashMap::new();

        for (state, prior_prob) in &outcomes {
            let new_dist_val = self.call_func(&func, std::slice::from_ref(state));
            let new_outcomes: Vec<(RuntimeValue, f64)> = match &new_dist_val {
                RuntimeValue::Dist(d) => self.get_dist_outcomes_dynamic(d),
                RuntimeValue::DynDist(o) => o.clone(),
                v => panic!(
                    "Transition function '{}' must return a distribution, got {}",
                    func_name, v
                ),
            };
            for (new_state, new_prob) in new_outcomes {
                let key = Self::dyn_key(&new_state);
                let entry = merged.entry(key).or_insert((new_state, 0.0));
                entry.1 += prior_prob * new_prob;
            }
        }

        let mut result: Vec<(RuntimeValue, f64)> =
            merged.into_values().collect();
        // Sort by display label for deterministic output.
        result.sort_by(|(a, _), (b, _)| format!("{}", a).cmp(&format!("{}", b)));
        RuntimeValue::DynDist(result)
    }

    /// Apply a Markov transition function `n` times starting from `initial_state`.
    /// Returns the marginal distribution over states after `n` steps.
    fn eval_step(
        &self,
        initial: RuntimeValue,
        func_name: &str,
        n: usize,
    ) -> RuntimeValue {
        // Start from a delta distribution concentrated on the initial state.
        let mut current = RuntimeValue::DynDist(vec![(initial, 1.0)]);
        for _ in 0..n {
            current = self.eval_bind(current, func_name);
        }
        current
    }

    /// Methods callable on a `DynDist` value (`:visualise()`, `:sample()`).
    fn eval_dyn_dist_method(
        &self,
        outcomes: Vec<(RuntimeValue, f64)>,
        method: &str,
        _args: &[Expr],
    ) -> RuntimeValue {
        match method {
            "visualise" | "visualize" => {
                // Merge by display key (should already be merged, but be safe).
                let mut merged: HashMap<String, f64> = HashMap::new();
                for (state, prob) in &outcomes {
                    *merged.entry(format!("{}", state)).or_insert(0.0) += prob;
                }
                let mut bars: Vec<(String, f64, String)> = merged
                    .into_iter()
                    .map(|(label, prob)| {
                        let display = format!("{:.4}", prob);
                        (label, prob, display)
                    })
                    .collect();
                // Sort alphabetically so output is deterministic.
                bars.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));

                // Use the first outcome to derive a label for the histogram.
                let type_name = outcomes.first().and_then(|(v, _)| {
                    if let RuntimeValue::EnumVariant(t, _) = v { Some(t.clone()) } else { None }
                }).unwrap_or_else(|| "State".to_string());
                let label = format!("Dist<{}>", type_name);

                RuntimeValue::Visualisation(HistogramData {
                    label,
                    kind: HistKind::Discrete,
                    bars,
                })
            }
            "sample" => {
                let mut rng = rand::thread_rng();
                let r: f64 = rng.r#gen();
                let mut cumulative = 0.0;
                for (state, prob) in &outcomes {
                    cumulative += prob;
                    if r < cumulative {
                        return state.clone();
                    }
                }
                outcomes
                    .last()
                    .map(|(s, _)| s.clone())
                    .unwrap_or(RuntimeValue::Int(0))
            }
            _ => panic!("Unknown method '{}' on dynamic distribution", method),
        }
    }

    // ── Distribution Methods ──────────────────────────────────────────────────

    fn eval_dist_method(&self, dist: &Dist, method: &str, args: &[Expr]) -> RuntimeValue {
        match method {
            "sample" => self.sample_dist(dist),

            "visualise" | "visualize" => {
                RuntimeValue::Visualisation(self.build_histogram_data(dist))
            }

            "expect" => {
                if args.len() != 1 {
                    panic!("expect() requires exactly 1 argument");
                }
                let expected = self.eval_expr(&args[0]).as_f64() as i64;
                let prob = self.prob_of_outcome(dist, expected);
                RuntimeValue::Frac(prob)
            }

            "min" => match dist {
                Dist::Uniform(a, _) | Dist::UniformContinuous(a, _) => self.eval_expr(a),
                _ => panic!("min() is only supported for uniform distributions"),
            },

            "max" => match dist {
                Dist::Uniform(_, b) | Dist::UniformContinuous(_, b) => self.eval_expr(b),
                _ => panic!("max() is only supported for uniform distributions"),
            },

            "mean" => match dist {
                Dist::Uniform(a, b) | Dist::UniformContinuous(a, b) => {
                    let av = self.eval_expr(a).as_f64();
                    let bv = self.eval_expr(b).as_f64();
                    RuntimeValue::Float((av + bv) / 2.0)
                }
                Dist::Bernoulli(p) => self.eval_expr(p),
                Dist::Binomial(n, p) => {
                    let nv = self.eval_expr(n).as_f64();
                    let pv = self.eval_expr(p).as_f64();
                    RuntimeValue::Float(nv * pv)
                }
                Dist::Geometric(p) => {
                    let pv = self.eval_expr(p).as_f64();
                    RuntimeValue::Float(1.0 / pv)
                }
                Dist::Beta(alpha, beta) => {
                    let a = self.eval_expr(alpha).as_f64();
                    let b = self.eval_expr(beta).as_f64();
                    RuntimeValue::Float(a / (a + b))
                }
                _ => panic!("mean() is not supported for this distribution type"),
            },

            _ => panic!("Unknown distribution method: '{}'", method),
        }
    }

    /// Probability of getting exactly `target` from a distribution (exact rational).
    fn prob_of_outcome(&self, dist: &Dist, target: i64) -> Fraction {
        match dist {
            Dist::Uniform(a_expr, b_expr) => {
                let a = self.eval_expr(a_expr).as_f64() as i64;
                let b = self.eval_expr(b_expr).as_f64() as i64;
                if target < a || target > b {
                    Fraction::from(0u64)
                } else {
                    Fraction::new(1u64, (b - a + 1) as u64)
                }
            }
            Dist::UniformContinuous(_, _) => {
                panic!("expect() is not supported for continuous distributions");
            }
            Dist::Discrete(pairs) => {
                for (val_expr, prob_expr) in pairs {
                    if self.eval_expr(val_expr).as_f64() as i64 == target {
                        return Self::float_to_frac(self.eval_expr(prob_expr).as_f64());
                    }
                }
                Fraction::from(0u64)
            }
            Dist::CombinedDist(d1, d2) => {
                let outcomes1 = self.get_dist_outcomes(d1);
                let outcomes2 = self.get_dist_outcomes(d2);
                let mut prob = Fraction::from(0u64);
                for (v1, p1) in &outcomes1 {
                    for (v2, p2) in &outcomes2 {
                        if v1 + v2 == target {
                            prob = prob + p1.clone() * p2.clone();
                        }
                    }
                }
                prob
            }
            Dist::ChainDist(_, _, _) => panic!("ChainDist probability calculation not implemented"),
            Dist::Bernoulli(p_expr) => {
                let p = Self::float_to_frac(self.eval_expr(p_expr).as_f64());
                match target {
                    1 => p,
                    0 => Fraction::from(1u64) - p,
                    _ => Fraction::from(0u64),
                }
            }
            Dist::Binomial(n_expr, p_expr) => {
                let n = self.eval_expr(n_expr).as_f64() as u64;
                let p = Self::float_to_frac(self.eval_expr(p_expr).as_f64());
                if target < 0 || target as u64 > n { return Fraction::from(0u64); }
                let k = target as u64;
                let binom = Fraction::from(binom_coeff(n, k));
                let p_k = Self::pow_frac(p.clone(), k);
                let q_nk = Self::pow_frac(Fraction::from(1u64) - p, n - k);
                binom * p_k * q_nk
            }
            Dist::Geometric(p_expr) => {
                if target < 1 { return Fraction::from(0u64); }
                // Use f64 to avoid u64 overflow; round to 4 sig figs for clean fractions.
                let p_f64 = self.eval_expr(p_expr).as_f64();
                let prob_f64 = (1.0 - p_f64).powi(target as i32 - 1) * p_f64;
                Self::float_to_frac(Self::round_sig(prob_f64, 4))
            }
            Dist::Beta(_, _) => {
                panic!("Beta distribution is continuous; expect() is not supported")
            }
        }
    }

    /// Returns (outcome_value, probability) pairs for analytical computation (exact rationals).
    fn get_dist_outcomes(&self, dist: &Dist) -> Vec<(i64, Fraction)> {
        match dist {
            Dist::Uniform(a_expr, b_expr) => {
                let a = self.eval_expr(a_expr).as_f64() as i64;
                let b = self.eval_expr(b_expr).as_f64() as i64;
                let prob = Fraction::new(1u64, (b - a + 1) as u64);
                (a..=b).map(|v| (v, prob.clone())).collect()
            }
            Dist::UniformContinuous(_, _) => {
                panic!("Continuous distributions cannot be combined analytically")
            }
            Dist::Discrete(pairs) => pairs
                .iter()
                .map(|(v, p)| {
                    (
                        self.eval_expr(v).as_f64() as i64,
                        Self::float_to_frac(self.eval_expr(p).as_f64()),
                    )
                })
                .collect(),
            Dist::CombinedDist(d1, d2) => {
                let o1 = self.get_dist_outcomes(d1);
                let o2 = self.get_dist_outcomes(d2);
                let mut out = Vec::new();
                for (v1, p1) in &o1 {
                    for (v2, p2) in &o2 {
                        out.push((v1 + v2, p1.clone() * p2.clone()));
                    }
                }
                out
            }
            Dist::ChainDist(_, _, _) => panic!("ChainDist not implemented"),
            Dist::Beta(_, _) => panic!("Beta distribution is continuous; cannot enumerate discrete outcomes"),
            Dist::Bernoulli(p_expr) => {
                let p = Self::float_to_frac(self.eval_expr(p_expr).as_f64());
                let q = Fraction::from(1u64) - p.clone();
                vec![(1, p), (0, q)]
            }
            Dist::Binomial(n_expr, p_expr) => {
                let n = self.eval_expr(n_expr).as_f64() as u64;
                let p = Self::float_to_frac(self.eval_expr(p_expr).as_f64());
                (0..=n)
                    .map(|k| {
                        let binom = Fraction::from(binom_coeff(n, k));
                        let p_k = Self::pow_frac(p.clone(), k);
                        let q_nk = Self::pow_frac(Fraction::from(1u64) - p.clone(), n - k);
                        (k as i64, binom * p_k * q_nk)
                    })
                    .collect()
            }
            Dist::Geometric(p_expr) => {
                // Compute probabilities in f64 to avoid u64 overflow in the Fraction
                // representation: (q/p denominator)^k grows as 10^k and exceeds u64::MAX
                // for moderate k (e.g. p=0.3 → overflow at k=20).
                // Round each probability to 4 significant figures before converting to a
                // Fraction so that floating-point noise doesn't produce ugly denominators.
                let p_f64 = self.eval_expr(p_expr).as_f64();
                let q_f64 = 1.0 - p_f64;
                let limit = if q_f64 <= 0.0 {
                    1i64
                } else {
                    (0.001_f64.ln() / q_f64.ln()).ceil() as i64
                };
                let mut outcomes = Vec::new();
                let mut prob_f64 = p_f64;
                for k in 1..=limit {
                    outcomes.push((k, Self::float_to_frac(Self::round_sig(prob_f64, 4))));
                    prob_f64 *= q_f64;
                }
                outcomes
            }
        }
    }

    // ── Histogram Construction ────────────────────────────────────────────────

    /// Build a `HistogramData` from any distribution for use by `:visualise()`.
    fn build_histogram_data(&self, dist: &Dist) -> HistogramData {
        let label = format_dist(dist);
        match dist {
            Dist::Beta(alpha_expr, beta_expr) => {
                let alpha = self.eval_expr(alpha_expr).as_f64();
                let beta_val = self.eval_expr(beta_expr).as_f64();
                // Discretize the Beta PDF into 20 bins at midpoints 0.025, 0.075, ..., 0.975
                let n_bins = 20usize;
                let mut raw: Vec<(f64, f64)> = (0..n_bins)
                    .map(|i| {
                        let p = (i as f64 + 0.5) / n_bins as f64;
                        // Unnormalized Beta PDF: p^(α-1) * (1-p)^(β-1)
                        let pdf = p.powf(alpha - 1.0) * (1.0 - p).powf(beta_val - 1.0);
                        (p, pdf)
                    })
                    .collect();
                let total: f64 = raw.iter().map(|(_, v)| v).sum();
                let bars = raw
                    .iter_mut()
                    .map(|(p, pdf)| {
                        let prob = if total > 0.0 { *pdf / total } else { 1.0 / n_bins as f64 };
                        let display = format!("{:.4}", prob);
                        (format!("{:.2}", p), prob, display)
                    })
                    .collect();
                HistogramData { label, kind: HistKind::Discrete, bars }
            }
            Dist::UniformContinuous(a_expr, b_expr) => {
                let min = self.eval_expr(a_expr).as_f64();
                let max = self.eval_expr(b_expr).as_f64();
                HistogramData {
                    label,
                    kind: HistKind::Continuous { min, max, mean: (min + max) / 2.0 },
                    bars: vec![],
                }
            }
            _ => {
                // Merge outcomes (CombinedDist can produce duplicate keys)
                let outcomes = self.get_dist_outcomes(dist);
                let mut merged: HashMap<i64, Fraction> = HashMap::new();
                for (v, p) in outcomes {
                    let entry = merged.entry(v).or_insert_with(|| Fraction::from(0u64));
                    *entry = entry.clone() + p;
                }
                let mut bars: Vec<(String, f64, String)> = merged
                    .into_iter()
                    .map(|(v, p)| {
                        let f = p.to_f64().unwrap_or(0.0);
                        let display = format!("{}", p);
                        (v.to_string(), f, display)
                    })
                    .collect();
                bars.sort_by_key(|(k, _, _)| k.parse::<i64>().unwrap_or(0));
                HistogramData { label, kind: HistKind::Discrete, bars }
            }
        }
    }

    // ── Sampling ──────────────────────────────────────────────────────────────

    fn sample_dist(&self, dist: &Dist) -> RuntimeValue {
        let mut rng = rand::thread_rng();
        self.sample_dist_with(&mut rng, dist)
    }

    fn sample_dist_with<R: Rng>(&self, rng: &mut R, dist: &Dist) -> RuntimeValue {
        match dist {
            Dist::Uniform(a_expr, b_expr) => {
                let a = self.eval_expr(a_expr).as_f64() as i64;
                let b = self.eval_expr(b_expr).as_f64() as i64;
                RuntimeValue::Int(rng.gen_range(a..=b))
            }
            Dist::UniformContinuous(a_expr, b_expr) => {
                let a = self.eval_expr(a_expr).as_f64();
                let b = self.eval_expr(b_expr).as_f64();
                RuntimeValue::Float(rng.gen_range(a..b))
            }
            Dist::Bernoulli(p_expr) => {
                let p = self.eval_expr(p_expr).as_f64();
                RuntimeValue::Bool(rng.gen_bool(p))
            }
            Dist::Binomial(n_expr, p_expr) => {
                let n = self.eval_expr(n_expr).as_f64() as u64;
                let p = self.eval_expr(p_expr).as_f64();
                let count = (0..n).filter(|_| rng.gen_bool(p)).count() as i64;
                RuntimeValue::Int(count)
            }
            Dist::Geometric(p_expr) => {
                let p = self.eval_expr(p_expr).as_f64();
                let mut count = 1i64;
                while !rng.gen_bool(p) {
                    count += 1;
                }
                RuntimeValue::Int(count)
            }
            Dist::Discrete(pairs) => {
                let evaluated: Vec<(f64, f64)> = pairs
                    .iter()
                    .map(|(v, p)| (self.eval_expr(v).as_f64(), self.eval_expr(p).as_f64()))
                    .collect();
                let r: f64 = rng.r#gen();
                let mut cumulative = 0.0;
                for (val, prob) in &evaluated {
                    cumulative += prob;
                    if r < cumulative {
                        if val.fract() == 0.0 {
                            return RuntimeValue::Int(*val as i64);
                        } else {
                            return RuntimeValue::Float(*val);
                        }
                    }
                }
                let (val, _) = evaluated.last().unwrap();
                if val.fract() == 0.0 { RuntimeValue::Int(*val as i64) } else { RuntimeValue::Float(*val) }
            }
            Dist::CombinedDist(d1, d2) => {
                let s1 = self.sample_dist_with(rng, d1);
                let s2 = self.sample_dist_with(rng, d2);
                match (s1, s2) {
                    (RuntimeValue::Int(a), RuntimeValue::Int(b)) => RuntimeValue::Int(a + b),
                    (RuntimeValue::Float(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a + b),
                    (RuntimeValue::Int(a), RuntimeValue::Float(b)) => RuntimeValue::Float(a as f64 + b),
                    (RuntimeValue::Float(a), RuntimeValue::Int(b)) => RuntimeValue::Float(a + b as f64),
                    (a, b) => panic!("Cannot combine distribution samples {:?} and {:?}", a, b),
                }
            }
            Dist::ChainDist(_, _, _) => panic!("ChainDist sampling not implemented"),
            Dist::Beta(alpha_expr, beta_expr) => {
                let alpha = self.eval_expr(alpha_expr).as_f64();
                let beta  = self.eval_expr(beta_expr).as_f64();
                RuntimeValue::Float(sample_beta(rng, alpha, beta))
            }
        }
    }

    // ── Built-in & User Function Calls ────────────────────────────────────────

    fn eval_func_call(&self, name: &str, args: &[Expr]) -> RuntimeValue {
        // ── Markov chain special forms ────────────────────────────────────────
        // These are handled before evaluating args because the second argument
        // is a function name (an identifier), not a value.

        if name == "bind" {
            if args.len() != 2 {
                panic!("bind() requires 2 arguments: bind(distribution, transition_function)");
            }
            let dist_val = self.eval_expr(&args[0]);
            let func_name = match &args[1] {
                Expr::Var(n) => n.clone(),
                _ => panic!("bind() second argument must be a transition function name"),
            };
            return self.eval_bind(dist_val, &func_name);
        }

        if name == "step" {
            if args.len() != 3 {
                panic!("step() requires 3 arguments: step(initial_state, transition_function, n)");
            }
            let initial = self.eval_expr(&args[0]);
            let func_name = match &args[1] {
                Expr::Var(n) => n.clone(),
                _ => panic!("step() second argument must be a transition function name"),
            };
            let n = self.eval_expr(&args[2]).as_f64() as usize;
            return self.eval_step(initial, &func_name, n);
        }

        let eval_args: Vec<RuntimeValue> = args.iter().map(|a| self.eval_expr(a)).collect();

        match name {
            // ── Built-ins ─────────────────────────────────────────────────────
            "jacobi" => {
                if eval_args.len() != 2 {
                    panic!("jacobi() requires 2 arguments");
                }
                RuntimeValue::Int(jacobi_symbol(
                    eval_args[0].as_f64() as i64,
                    eval_args[1].as_f64() as i64,
                ))
            }
            "mod_exp" => {
                if eval_args.len() != 3 {
                    panic!("mod_exp() requires 3 arguments");
                }
                RuntimeValue::Int(mod_exp(
                    eval_args[0].as_f64() as i64,
                    eval_args[1].as_f64() as i64,
                    eval_args[2].as_f64() as i64,
                ))
            }
            // ── User-defined regular functions ────────────────────────────────
            _ => {
                if self.pb_funcs.contains_key(name) {
                    panic!(
                        "'{}' is a probabilistic function; call it with \
                         `let x, info = {}(args) with confidence >= val`",
                        name, name
                    );
                }
                let func = self
                    .funcs
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined function: '{}'", name));
                self.call_func(&func, &eval_args)
            }
        }
    }

    fn call_func(&self, func: &FuncDef, args: &[RuntimeValue]) -> RuntimeValue {
        if args.len() != func.params.len() {
            panic!(
                "Function '{}' expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            );
        }
        let mut env = self.new_child();
        for (param, arg) in func.params.iter().zip(args.iter()) {
            env.vars.insert(param.name.clone(), arg.clone());
        }
        for stmt in &func.body {
            match env.exec_stmt(stmt) {
                FlowControl::Return(val) => return val,
                FlowControl::Continue => {}
            }
        }
        RuntimeValue::Int(0) // implicit return 0 if no return statement
    }

    // ── Probabilistic Function Execution ─────────────────────────────────────

    /// Execute a pb function body once, returning Certain(v) or Uncertain(v).
    fn call_pb_func_once(&self, func: &PbFuncDef, args: &[RuntimeValue]) -> RuntimeValue {
        if args.len() != func.params.len() {
            panic!(
                "Pb function '{}' expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            );
        }
        let mut env = self.new_child();
        for (param, arg) in func.params.iter().zip(args.iter()) {
            env.vars.insert(param.name.clone(), arg.clone());
        }
        for stmt in &func.body {
            match env.exec_stmt(stmt) {
                FlowControl::Return(val) => return val,
                FlowControl::Continue => {}
            }
        }
        panic!("Pb function '{}' did not return a value", func.name);
    }

    /// Execute a pb function for the required number of rounds to meet `target_confidence`.
    /// Returns `(result_value, info_value)`.
    fn call_pb_func(
        &self,
        func: &PbFuncDef,
        args: &[RuntimeValue],
        target_confidence: f64,
    ) -> (RuntimeValue, RuntimeValue) {
        let rounds_needed = compute_rounds_needed(&func.error_class, target_confidence);

        match func.error_class {
            ErrorClass::RP => self.run_rp_rounds(func, args, rounds_needed),
            ErrorClass::CoRP => self.run_corp_rounds(func, args, rounds_needed),
            ErrorClass::BPP => self.run_bpp_rounds(func, args, rounds_needed),
        }
    }

    /// RP: stop early on `Certain(v)` (definitive answer); accumulate `Uncertain(v)`.
    /// If all k rounds return `Uncertain`, confidence = 1 - (1/2)^k.
    fn run_rp_rounds(
        &self,
        func: &PbFuncDef,
        args: &[RuntimeValue],
        max_rounds: u64,
    ) -> (RuntimeValue, RuntimeValue) {
        let mut last_uncertain_val: Option<RuntimeValue> = None;

        for round in 1..=max_rounds {
            match self.call_pb_func_once(func, args) {
                RuntimeValue::Certain(val) => {
                    let info = RuntimeValue::Info { rounds: round, confidence: 1.0 };
                    return (*val, info);
                }
                RuntimeValue::Uncertain(val) => {
                    last_uncertain_val = Some(*val);
                }
                v => panic!(
                    "Pb function '{}' must return Certain(v) or Uncertain(v), got {:?}",
                    func.name, v
                ),
            }
        }

        let val = last_uncertain_val
            .unwrap_or_else(|| panic!("Pb function '{}' returned no value", func.name));
        let confidence = 1.0 - 0.5_f64.powi(max_rounds as i32);
        let info = RuntimeValue::Info { rounds: max_rounds, confidence };
        (val, info)
    }

    /// coRP: stop early on `Certain(v)` (definitive answer); accumulate `Uncertain(v)`.
    /// Mirror of RP — the certain answer signals a definitive result.
    fn run_corp_rounds(
        &self,
        func: &PbFuncDef,
        args: &[RuntimeValue],
        max_rounds: u64,
    ) -> (RuntimeValue, RuntimeValue) {
        let mut last_uncertain_val: Option<RuntimeValue> = None;

        for round in 1..=max_rounds {
            match self.call_pb_func_once(func, args) {
                RuntimeValue::Certain(val) => {
                    let info = RuntimeValue::Info { rounds: round, confidence: 1.0 };
                    return (*val, info);
                }
                RuntimeValue::Uncertain(val) => {
                    last_uncertain_val = Some(*val);
                }
                v => panic!(
                    "Pb function '{}' must return Certain(v) or Uncertain(v), got {:?}",
                    func.name, v
                ),
            }
        }

        let val = last_uncertain_val
            .unwrap_or_else(|| panic!("Pb function '{}' returned no value", func.name));
        let confidence = 1.0 - 0.5_f64.powi(max_rounds as i32);
        let info = RuntimeValue::Info { rounds: max_rounds, confidence };
        (val, info)
    }

    /// BPP: run all k rounds, take the majority vote.
    /// Confidence bound (Chernoff, assuming per-round success prob p = 3/4):
    ///   P(error) ≈ exp(-2k(p - 1/2)²) = exp(-k/8)
    fn run_bpp_rounds(
        &self,
        func: &PbFuncDef,
        args: &[RuntimeValue],
        rounds: u64,
    ) -> (RuntimeValue, RuntimeValue) {
        let mut true_votes = 0u64;
        let mut false_votes = 0u64;

        for _ in 0..rounds {
            let inner = match self.call_pb_func_once(func, args) {
                RuntimeValue::Certain(v) | RuntimeValue::Uncertain(v) => *v,
                v => panic!(
                    "Pb function '{}' must return Certain(v) or Uncertain(v), got {:?}",
                    func.name, v
                ),
            };
            match inner {
                RuntimeValue::Bool(true) => true_votes += 1,
                RuntimeValue::Bool(false) => false_votes += 1,
                v => panic!("BPP function round returned non-boolean inner value: {:?}", v),
            }
        }

        let majority = RuntimeValue::Bool(true_votes > false_votes);
        // Chernoff: exp(-k/8) with p = 3/4
        let error_prob = (-(rounds as f64) / 8.0).exp();
        let info = RuntimeValue::Info { rounds, confidence: 1.0 - error_prob };
        (majority, info)
    }

    // ── Statement Execution ───────────────────────────────────────────────────

    fn exec_stmt(&mut self, stmt: &Statement) -> FlowControl {
        match stmt {
            Statement::Decl(expr) => {
                if let Expr::Var(name) = expr {
                    self.vars.insert(name.clone(), RuntimeValue::Int(0));
                } else {
                    panic!("Declaration must be a variable name");
                }
                FlowControl::Continue
            }

            Statement::Assign { name, value } | Statement::DeclAssign { name, value } => {
                let var_name = match name {
                    Expr::Var(n) => n.clone(),
                    _ => panic!("Left-hand side of assignment must be a variable name"),
                };
                let val = self.eval_expr(value);
                self.vars.insert(var_name, val);
                FlowControl::Continue
            }

            Statement::HardcodedOutput(expr) => {
                let val = self.eval_expr(expr);
                match val {
                    RuntimeValue::Visualisation(data) => {
                        self.output.push(OutputLine::Hist(data));
                    }
                    other => {
                        self.output.push(OutputLine::Text(format!("{}", other)));
                    }
                }
                FlowControl::Continue
            }

            Statement::Return(maybe_expr) => {
                let val = match maybe_expr {
                    Some(e) => self.eval_expr(e),
                    None => RuntimeValue::Int(0),
                };
                FlowControl::Return(val)
            }

            Statement::If { cond, then_block, else_block } => {
                let cond_val = self.eval_expr(cond).as_bool();
                let block = if cond_val {
                    Some(then_block.as_slice())
                } else {
                    else_block.as_deref()
                };
                if let Some(stmts) = block {
                    for s in stmts {
                        match self.exec_stmt(s) {
                            FlowControl::Return(v) => return FlowControl::Return(v),
                            FlowControl::Continue => {}
                        }
                    }
                }
                FlowControl::Continue
            }

            Statement::PbCallAssign { result_var, info_var, func_name, args, confidence } => {
                let func = self
                    .pb_funcs
                    .get(func_name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined probabilistic function: '{}'", func_name));
                let eval_args: Vec<RuntimeValue> =
                    args.iter().map(|a| self.eval_expr(a)).collect();
                let (result, info) = self.call_pb_func(&func, &eval_args, *confidence);
                self.vars.insert(result_var.clone(), result);
                self.vars.insert(info_var.clone(), info);
                FlowControl::Continue
            }

            Statement::MapCallAssign { var, func_name, array_expr, confidence } => {
                let arr = match self.eval_expr(array_expr) {
                    RuntimeValue::Array(elems) => elems,
                    other => panic!(
                        "map() second argument must be an array, got {:?}",
                        other
                    ),
                };
                let n = arr.len();

                let results: Vec<RuntimeValue> = if let Some(conf) = confidence {
                    // Union bound: split error budget evenly across all n elements.
                    //   P(any element wrong) <= sum of per-element errors = n * (1-per_conf)
                    //   Setting n * (1-per_conf) = 1-conf gives per_conf = 1 - (1-conf)/n.
                    let per_conf = if n == 0 {
                        *conf
                    } else {
                        1.0 - (1.0 - conf) / n as f64
                    };
                    let pb_func = self
                        .pb_funcs
                        .get(func_name)
                        .cloned()
                        .unwrap_or_else(|| {
                            if self.funcs.contains_key(func_name) {
                                panic!(
                                    "'{}' is a regular function; \
                                     'with confidence' only applies to pb functions",
                                    func_name
                                )
                            } else {
                                panic!("Undefined function: '{}'", func_name)
                            }
                        });
                    let mut out = Vec::with_capacity(n);
                    for elem in arr {
                        let (result, _info) =
                            self.call_pb_func(&pb_func, &[elem], per_conf);
                        out.push(result);
                    }
                    out
                } else {
                    // Regular function: apply element-wise, no confidence needed.
                    let func = self
                        .funcs
                        .get(func_name)
                        .cloned()
                        .unwrap_or_else(|| {
                            if self.pb_funcs.contains_key(func_name) {
                                panic!(
                                    "'{}' is a probabilistic function; \
                                     use 'with confidence >= ...' when mapping it",
                                    func_name
                                )
                            } else {
                                panic!("Undefined function: '{}'", func_name)
                            }
                        });
                    let mut out = Vec::with_capacity(n);
                    for elem in arr {
                        out.push(self.call_func(&func, &[elem]));
                    }
                    out
                };

                self.vars.insert(var.clone(), RuntimeValue::Array(results));
                FlowControl::Continue
            }

            Statement::DistributionOf { var, func_name, args, mode } => {
                let func = self
                    .pb_funcs
                    .get(func_name)
                    .cloned()
                    .unwrap_or_else(|| panic!(
                        "distribution_of: '{}' is not a probabilistic function", func_name
                    ));
                let eval_args: Vec<RuntimeValue> =
                    args.iter().map(|a| self.eval_expr(a)).collect();

                let dist_val = match mode {
                    DistributionOfMode::Analytical => {
                        // Derive the per-round distribution from the error class alone.
                        // RP / coRP: per-round probability of Certain ≥ 1/2
                        //   → worst-case rounds-until-Certain ~ Geometric(0.5)
                        // BPP: per-round vote correctness ≥ 3/4
                        //   → model each vote as Bernoulli(0.75)
                        let dist = match func.error_class {
                            ErrorClass::RP | ErrorClass::CoRP => {
                                Dist::Geometric(Box::new(Expr::Float(0.5)))
                            }
                            ErrorClass::BPP => {
                                Dist::Bernoulli(Box::new(Expr::Float(0.75)))
                            }
                        };
                        RuntimeValue::Dist(dist)
                    }

                    DistributionOfMode::Empirical(n) => {
                        // Run N single rounds and report empirical Certain frequency.
                        let mut n_certain = 0i64;
                        for _ in 0..*n {
                            match self.call_pb_func_once(&func, &eval_args) {
                                RuntimeValue::Certain(_) => n_certain += 1,
                                RuntimeValue::Uncertain(_) => {}
                                v => panic!(
                                    "distribution_of: pb function must return \
                                     Certain(v) or Uncertain(v), got {:?}", v
                                ),
                            }
                        }
                        let p = if *n > 0 { n_certain as f64 / *n as f64 } else { 0.0 };
                        RuntimeValue::Dist(Dist::Bernoulli(Box::new(Expr::Float(p))))
                    }

                    DistributionOfMode::Bayesian(n) => {
                        // Run N rounds, update Beta(1,1) prior with observations.
                        // Posterior: Beta(1 + n_certain, 1 + n_uncertain)
                        let mut n_certain = 0i64;
                        let mut n_uncertain = 0i64;
                        for _ in 0..*n {
                            match self.call_pb_func_once(&func, &eval_args) {
                                RuntimeValue::Certain(_) => n_certain += 1,
                                RuntimeValue::Uncertain(_) => n_uncertain += 1,
                                v => panic!(
                                    "distribution_of: pb function must return \
                                     Certain(v) or Uncertain(v), got {:?}", v
                                ),
                            }
                        }
                        let alpha = 1.0 + n_certain as f64;
                        let beta_val = 1.0 + n_uncertain as f64;
                        RuntimeValue::Dist(Dist::Beta(
                            Box::new(Expr::Float(alpha)),
                            Box::new(Expr::Float(beta_val)),
                        ))
                    }
                };

                self.vars.insert(var.clone(), dist_val);
                FlowControl::Continue
            }
        }
    }
}

// ── Pure Maths Helpers ─────────────────────────────────────────────────

/// Iterative Jacobi symbol (a/n). Returns -1, 0, or 1.
/// n must be a positive integer. If n is even (or 1) the symbol is not
/// classically defined, but we return 0 — this signals compositeness to
/// the Solovay-Strassen algorithm, producing the correct `Certain(false)`.
fn jacobi_symbol(mut a: i64, mut n: i64) -> i64 {
    assert!(n > 0, "jacobi: n must be positive (got n={})", n);
    // Even n is composite; return 0 so the caller detects a non-prime.
    if n % 2 == 0 {
        return 0;
    }
    a = a.rem_euclid(n);
    let mut result = 1i64;
    while a != 0 {
        while a % 2 == 0 {
            a /= 2;
            match n % 8 {
                3 | 5 => result = -result,
                _ => {}
            }
        }
        std::mem::swap(&mut a, &mut n);
        if a % 4 == 3 && n % 4 == 3 {
            result = -result;
        }
        a = a.rem_euclid(n);
    }
    if n == 1 { result } else { 0 }
}

/// Modular exponentiation: base^exp mod modulus (all must be non-negative).
fn mod_exp(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1i64;
    base = base.rem_euclid(modulus);
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % modulus;
        }
        exp /= 2;
        base = base * base % modulus;
    }
    result
}

/// Minimum rounds needed to achieve `target_confidence`.
fn compute_rounds_needed(error_class: &ErrorClass, target_confidence: f64) -> u64 {
    let error = 1.0 - target_confidence;
    let k = match error_class {
        // RP / coRP: per-round error ≤ 1/2 (Geometric decay).
        // Need (1/2)^k ≤ error  →  k ≥ −log₂(error)
        ErrorClass::RP | ErrorClass::CoRP => (-error.log2()).ceil(),
        // BPP: majority-vote + Chernoff bound (p = 3/4 assumed).
        // Need exp(−k/8) ≤ error  →  k ≥ −8·ln(error)
        ErrorClass::BPP => (-8.0 * error.ln()).ceil(),
    };
    (k as u64).max(1)
}

/// Sample Gamma(n, 1) for integer n using the sum-of-exponentials identity:
///   Gamma(n, 1) = -ln(U₁ · U₂ · … · Uₙ) = sum of n i.i.d. Exp(1) values.
fn sample_gamma_int<R: rand::Rng>(rng: &mut R, n: u64) -> f64 {
    (0..n.max(1)).map(|_| -rng.r#gen::<f64>().ln()).sum()
}

/// Sample from Beta(alpha, beta) using the Gamma relationship:
///   X = Ga / (Ga + Gb)  where Ga ~ Gamma(alpha, 1), Gb ~ Gamma(beta, 1).
/// Works exactly for integer alpha/beta (as produced by the Bayesian update).
fn sample_beta<R: rand::Rng>(rng: &mut R, alpha: f64, beta: f64) -> f64 {
    let a = alpha.round().max(1.0) as u64;
    let b = beta.round().max(1.0) as u64;
    let ga = sample_gamma_int(rng, a);
    let gb = sample_gamma_int(rng, b);
    let total = ga + gb;
    if total == 0.0 { 0.5 } else { (ga / total).clamp(0.0, 1.0) }
}

/// Binomial coefficient C(n, k).
fn binom_coeff(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k); // use symmetry
    let mut result = 1u64;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    result
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Execute all program items and return the output lines.
/// Shared by `run`, `run_to_string`, and `run_to_html`.
fn collect_output(items: &[ProgramItem]) -> Vec<OutputLine> {
    let mut env = RuntimeEnv::new();
    // Two-pass: register all definitions before executing statements,
    // so call-before-definition works.
    for item in items {
        match item {
            ProgramItem::FuncDef(f) => { env.funcs.insert(f.name.clone(), f.clone()); }
            ProgramItem::PbFuncDef(f) => { env.pb_funcs.insert(f.name.clone(), f.clone()); }
            ProgramItem::EnumDef(e) => { env.register_enum(e); }
            ProgramItem::Statement(_) => {}
        }
    }
    for item in items {
        if let ProgramItem::Statement(stmt) = item {
            env.exec_stmt(stmt);
        }
    }
    env.output
}

/// CLI mode: print output to stdout, rendering histograms as ASCII art.
pub fn run(items: &[ProgramItem]) {
    for line in collect_output(items) {
        match line {
            OutputLine::Text(s) => println!("{}", s),
            OutputLine::Hist(data) => print!("{}", visualiser::render_cli(&data)),
        }
    }
}

/// Return output as a plain string (for tests and `try_run_program`).
/// Histograms are rendered as ASCII art.
pub fn run_to_string(items: &[ProgramItem]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for line in collect_output(items) {
        match line {
            OutputLine::Text(s) => parts.push(s),
            OutputLine::Hist(data) => {
                let rendered = visualiser::render_cli(&data);
                parts.push(rendered.trim_end().to_string());
            }
        }
    }
    parts.join("\n")
}

/// Return output as an HTML string for the web playground.
/// Histograms are rendered as inline SVG; text is HTML-escaped inside `<pre>`.
pub fn run_to_html(items: &[ProgramItem]) -> String {
    let mut html = String::new();
    let mut text_buf: Vec<String> = Vec::new();
    let mut vis_idx: usize = 0;

    for line in collect_output(items) {
        match line {
            OutputLine::Text(s) => text_buf.push(s),
            OutputLine::Hist(data) => {
                // Flush accumulated text lines first
                if !text_buf.is_empty() {
                    html.push_str(r#"<pre class="yappl-text">"#);
                    html.push_str(&visualiser::html_esc(&text_buf.join("\n")));
                    html.push_str("</pre>");
                    text_buf.clear();
                }
                html.push_str(&visualiser::render_svg(&data, vis_idx));
                vis_idx += 1;
            }
        }
    }
    // Flush any remaining text
    if !text_buf.is_empty() {
        html.push_str(r#"<pre class="yappl-text">"#);
        html.push_str(&visualiser::html_esc(&text_buf.join("\n")));
        html.push_str("</pre>");
    }
    html
}

/// Run a YAPPL source string, returning Ok(output) or Err(error message).
/// Catches both parse errors and runtime panics.
pub fn try_run_program(source: &str) -> Result<String, String> {
    use std::panic::{self, AssertUnwindSafe};
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let items = crate::parser::parse(source);
        run_to_string(&items)
    }));
    match result {
        Ok(output) => Ok(output),
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "Unknown error".to_string());
            Err(msg)
        }
    }
}
