use std::collections::HashMap;

use crate::ast::*;

struct RuntimeEnv {
    vars: HashMap<String, f64>, // changed to f64 to support floating-point probabilities
    dists: HashMap<String, Dist>
}

impl RuntimeEnv {
    fn new() -> Self {
        RuntimeEnv { 
            vars: HashMap::new(),
            dists: HashMap::new()
        }
    }

    fn eval_expr(&self, expr: &Expr) -> f64 {
        match expr {
            Expr::Int(n) => *n as f64,

            Expr::Var(name) => {
                *self.vars.get(name)
                    .unwrap_or_else(|| panic!("Undefined variable: {}", name))
            }

            // Expr::FuncCall(name, args) => {
            //     let eval_args: Vec<i64> = args.iter().map(|e| self.eval_expr(e)).collect();
            //     self.eval_func_call(name, &eval_args)
            // }

            Expr::Neg(inner) => -self.eval_expr(inner),

            Expr::Add(a, b) => self.eval_expr(a) + self.eval_expr(b),
            Expr::Sub(a, b) => self.eval_expr(a) - self.eval_expr(b),
            Expr::Mul(a, b) => self.eval_expr(a) * self.eval_expr(b),
            Expr::Div(a, b) => self.eval_expr(a) / self.eval_expr(b),

            Expr::Dist(_) => panic!("Cannot evaluate Dist expression as f64"),
            Expr::DistMethodCall { .. } => panic!("Cannot evaluate DistMethodCall expression as f64"),
        }
    }

    pub fn exec_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Decl(expr) => {
                if let Expr::Var(name) = expr {
                    self.vars.insert(name.clone(), 0.0);
                } else {
                    panic!("Declaration must contain a variable name");
                }
            }

            Statement::Assign { name, value } => {
                let var_name = match name {
                    Expr::Var(n) => n.clone(),
                    _ => panic!("Left side of assignment must be a variable"),
                };

                match value {
                    Expr::Dist(dist) => {
                        self.dists.insert(var_name, dist.clone());
                    }
                    Expr::DistMethodCall { var, method, args } => {
                        let result = self.eval_dist_method(var, method, args);
                        self.vars.insert(var_name, result);
                    }
                    _ => {
                        let val = self.eval_expr(value);
                        self.vars.insert(var_name, val);
                    }
                }
            }

            Statement::DeclAssign { name, value } => {
                let var_name = match name {
                    Expr::Var(n) => n.clone(),
                    _ => panic!("Left side of assignment must be a variable"),
                };

                match value {
                    Expr::Dist(dist) => {
                        self.dists.insert(var_name, dist.clone());
                    }
                    Expr::DistMethodCall { var, method, args } => {
                        let result = self.eval_dist_method(var, method, args);
                        self.vars.insert(var_name, result);
                    }
                    _ => {
                        let val = self.eval_expr(value);
                        self.vars.insert(var_name, val);
                    }
                }
            }

            Statement::HardcodedOutput(expr) => {
                let v = self.eval_expr(expr);
                // Print with appropriate precision for floating-point numbers
                if v.fract() == 0.0 {
                    println!("{}", v as i64);
                } else {
                    println!("{}", v);
                }
            }
        }
    }

    fn eval_dist_method(&self, var: &str, method: &str, args: &[Expr]) -> f64 {
        let dist = self.dists.get(var)
            .unwrap_or_else(|| panic!("Undefined distribution: {}", var));

        match method {
            "expect" => {
                if args.len() != 1 {
                    panic!("expect method requires exactly 1 argument");
                }
                if let Expr::Int(_expected) = args[0] {
                    // Calculate probability of getting the expected value
                    match dist {
                        Dist::Uniform(a, b) => {
                            // For a uniform distribution, probability = 1 / (num_values)
                            let num_values = (b - a + 1) as f64;
                            1.0 / num_values
                        },
                        Dist::ChainDist(_, _, _) => {
                            panic!("ChainDist probability calculation not yet implemented");
                        }
                    }
                } else {
                    panic!("expect method argument must be an integer");
                }
            }
            _ => panic!("Unknown distribution method: {}", method),
        }
    }
}

pub fn print_statements(statement_list: Vec<Statement>){
    println!("Size: {}", statement_list.len());
    
    for stmt in statement_list {
        println!("Statement: {}", stmt);
    }
}

pub fn run(stmts: &[Statement]) {
    let mut env = RuntimeEnv::new();

    for stmt in stmts {
        env.exec_stmt(stmt);
    }
}