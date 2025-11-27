use crate::ast::*;

pub fn print_statements(statement_list: Vec<Statement>){
    println!("size: {}", statement_list.len());
    
    for stmt in statement_list {
        println!("AA: {}", stmt);
    }
}