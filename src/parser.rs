use std::str::FromStr;

use nom::{
  IResult,
  Parser,
  error::ParseError,
  sequence::{delimited, terminated},
  character::complete::multispace0,
  bytes::complete::{tag, take_while1},
  multi::many0,
  branch::{alt},
};

use crate::ast::*;

// utility
fn eat_whitespace<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

// grammar

fn parse_program(input: &str) -> IResult<&str, Vec<Statement>>{
    parse_statement_list(input)
}

fn parse_statement_list(input: &str) -> IResult<&str, Vec<Statement>>{
    many0(terminated(parse_statement, eat_whitespace(tag(";"))))(input)
}   

fn parse_statement(input: &str) -> IResult<&str, Statement>{
    alt((parse_var_declaration, parse_assignment)).parse(input)
}

fn parse_var_declaration(input: &str) -> IResult<&str, Expr>{

}

fn partial_parse_assignment(input: &str, variable_node: Expr) -> IResult<&str, Expr>{
    
}

fn parse_assignment(input: &str) -> IResult<&str, Statement>{

}

// expression parsing

/*
    additive_term ::= multiplicative_term additive_term_prime
    additive_term_prime ::= ("+"|"-") multiplicative_term additive_term_prime | epsilon

    if additive_term_prime is epsilon -> don't construct an ASTnode for expr
    if additive_term_prime is something -> construct an Expr<add>
*/
fn parse_expr(input: &str) -> IResult<&str, Expr>{
    let (input, mul_term) = parse_mul_term(input)?;
    let (input, add_term_prime) = parse_additive_term_prime(input)?;
    
}

fn parse_additive_term_prime() -> IResult<&str, Expr>{

}


/*
    multiplicative_term ::= unary_term multiplicative_term_prime
    multiplicative_term_prime ::= ("*"|"/") unary_term multiplicative_term_prime | epsilon
*/
fn parse_mul_term(input: &str) -> IResult<&str, Expr>{

}

/*
    unary_term ::= "-" unary_term | primary_term
*/

/*
    primary_term ::= "(" expr ")" 
    | VAR_NAME
    | INT_LIT
*/

// literals
fn parse_int_literal(input: &str) -> IResult<&str, Expr>{
    let (input, digits) = take_while1(nom::AsChar::is_dec_digit)(input)?;

    Ok((input, Statement::Expr(Expr::Int(i64::from_str(digits).unwrap()))))
}

fn parse_identifier(input: &str) -> IResult<&str, Expr>{
    take_while1(nom::AsChar::is_alphanum)(input)
}

/*
    Run should be given a program (as a string) to execute
*/
pub fn parse(input: &str) -> Vec<Statement>{
    match parse_program(input) {
        Ok((_inp, statements)) => statements,
        Err(e) => {
            panic!("{:#?}", e);
        }
    }
}