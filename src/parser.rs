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

fn parse_assignment(input: &str) -> IResult<&str, Expr>{

}

fn parse_int_literal(input: &str) -> IResult<&str, Expr>{
    let (input, digits) = take_while1(nom::AsChar::is_dec_digit)(input)?;

    Ok((input, Statement::Expr(Expr::Int(i64::from_str(digits).unwrap()))))
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