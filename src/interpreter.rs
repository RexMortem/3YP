use std::collections::HashMap;
use rand::Rng;

use crate::ast::*;

struct RuntimeEnv {
    vars: HashMap<String, i64>, // just testing on ints for now
    dists: HashMap<String, Dist>
}

impl RuntimeEnv {
    fn new() -> Self {
        RuntimeEnv { 
            vars: HashMap::new(),
            dists: HashMap::new()
        }
    }

    fn eval_expr(&self, expr: &Expr) -> i64 {
        match expr {
            Expr::Int(n) => *n,

            Expr::Var(name) => {
                *self.vars.get(name)
                    .unwrap_or_else(|| panic!("Undefined variable: {}", name))
            }

            Expr::FuncCall(name, args) => {
                let eval_args: Vec<i64> = args.iter().map(|e| self.eval_expr(e)).collect();
                self.eval_func_call(name, &eval_args)
            }
            
            Expr::Neg(inner) => -self.eval_expr(inner),

            Expr::Add(a, b) => self.eval_expr(a) + self.eval_expr(b),
            Expr::Sub(a, b) => self.eval_expr(a) - self.eval_expr(b),
            Expr::Mul(a, b) => self.eval_expr(a) * self.eval_expr(b),
            Expr::Div(a, b) => self.eval_expr(a) / self.eval_expr(b),
        }
    }

    pub fn exec_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Decl(expr) => {
                if let Expr::Var(name) = expr {
                    self.vars.insert(name.clone(), 0);
                } else {
                    panic!("Declaration must contain a variable name");
                }
            }

            Statement::Assign { name, value } => {
                let var_name = match name {
                    Expr::Var(n) => n.clone(),
                    _ => panic!("Left side of assignment must be a variable"),
                };

                let val = self.eval_expr(value);
                self.vars.insert(var_name, val);
            }

            Statement::DeclAssign { name, value } => {
                let var_name = match name {
                    Expr::Var(n) => n.clone(),
                    _ => panic!("Left side of assignment must be a variable"),
                };

                let val = self.eval_expr(value);
                self.vars.insert(var_name, val);
            }

            Statement::HardcodedOutput(expr) => {
                let v = self.eval_expr(expr);
                println!("{}", v);
            }
        }
    }
}

pub fn print_statements(statement_list: Vec<Statement>){
    println!("size: {}", statement_list.len());
    
    for stmt in statement_list {
        println!("AA: {}", stmt);
    }
}

pub fn run(stmts: &[Statement]) {
    let mut env = RuntimeEnv::new();

    for stmt in stmts {
        env.exec_stmt(stmt);
    }
}