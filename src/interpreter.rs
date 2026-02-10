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
            Expr::Float(n) => *n,

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
            Expr::DistMethodCall { var, method, args } => self.eval_dist_method(var, method, args),
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

                // Check if this is a distribution combination (dist + dist)
                if let Expr::Add(box1, box2) = value {
                    if let (Expr::Var(var1), Expr::Var(var2)) = (box1.as_ref(), box2.as_ref()) {
                        if let (Some(dist1), Some(dist2)) = (self.dists.get(var1), self.dists.get(var2)) {
                            let combined = Dist::CombinedDist(Box::new(dist1.clone()), Box::new(dist2.clone()));
                            self.dists.insert(var_name, combined);
                            return;
                        }
                    }
                }

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

                // Check if this is a distribution combination (dist + dist)
                if let Expr::Add(box1, box2) = value {
                    if let (Expr::Var(var1), Expr::Var(var2)) = (box1.as_ref(), box2.as_ref()) {
                        if let (Some(dist1), Some(dist2)) = (self.dists.get(var1), self.dists.get(var2)) {
                            let combined = Dist::CombinedDist(Box::new(dist1.clone()), Box::new(dist2.clone()));
                            self.dists.insert(var_name, combined);
                            return;
                        }
                    }
                }

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
                let expected = self.eval_expr(&args[0]) as i64;
                // Calculate probability of getting the expected value
                match dist {
                    Dist::Uniform(a_expr, b_expr) => {
                        // Evaluate the bound expressions
                        let a = self.eval_expr(a_expr) as i64;
                        let b = self.eval_expr(b_expr) as i64;
                        // Check if expected value is within range
                        if expected < a || expected > b {
                            return 0.0;
                        }
                        // For a uniform distribution, probability = 1 / (num_values)
                        let num_values = (b - a + 1) as f64;
                        1.0 / num_values
                    },
                    Dist::Discrete(pairs) => {
                        // Look for the expected value in the pairs
                        for (value_expr, prob_expr) in pairs {
                            let value = self.eval_expr(value_expr) as i64;
                            if value == expected {
                                return self.eval_expr(prob_expr);
                            }
                        }
                        // Value not found in discrete distribution
                        0.0
                    },
                    Dist::CombinedDist(dist1, dist2) => {
                        // Get all values and probabilities from both distributions
                        let pairs1 = self.get_dist_outcomes(dist1);
                        let pairs2 = self.get_dist_outcomes(dist2);

                        // Sum all probabilities where value1 + value2 = expected
                        let mut total_prob = 0.0;
                        for (val1, prob1) in &pairs1 {
                            for (val2, prob2) in &pairs2 {
                                if val1 + val2 == expected {
                                    total_prob += prob1 * prob2;
                                }
                            }
                        }
                        total_prob
                    },
                    Dist::ChainDist(_, _, _) => {
                        panic!("ChainDist probability calculation not yet implemented");
                    }
                }
            }
            _ => panic!("Unknown distribution method: {}", method),
        }
    }

    fn get_dist_outcomes(&self, dist: &Dist) -> Vec<(i64, f64)> {
        match dist {
            Dist::Uniform(a_expr, b_expr) => {
                let a = self.eval_expr(a_expr) as i64;
                let b = self.eval_expr(b_expr) as i64;
                let num_values = (b - a + 1) as f64;
                let prob = 1.0 / num_values;
                (a..=b).map(|val| (val, prob)).collect()
            },
            Dist::Discrete(pairs) => {
                pairs.iter().map(|(val_expr, prob_expr)| {
                    let val = self.eval_expr(val_expr) as i64;
                    let prob = self.eval_expr(prob_expr);
                    (val, prob)
                }).collect()
            },
            Dist::CombinedDist(dist1, dist2) => {
                let pairs1 = self.get_dist_outcomes(dist1);
                let pairs2 = self.get_dist_outcomes(dist2);

                let mut result = vec![];
                for (val1, prob1) in &pairs1 {
                    for (val2, prob2) in &pairs2 {
                        result.push((val1 + val2, prob1 * prob2));
                    }
                }
                result
            },
            Dist::ChainDist(_, _, _) => {
                panic!("ChainDist not yet implemented");
            }
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