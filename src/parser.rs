use std::str::FromStr;

use nom::{
  IResult,
  Parser,
  error::ParseError,
  sequence::{delimited, terminated, pair, preceded},
  character::complete::multispace0,
  bytes::complete::{tag, take_while1},
  multi::many0,
  branch::{alt},
  combinator::{opt}
};

use crate::ast::*;

// utility
fn eat_ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
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
    many0(terminated(parse_statement, eat_ws(tag(";"))))(input)
}   

fn parse_statement(input: &str) -> IResult<&str, Statement>{
    alt((parse_var_declaration, parse_statement_assignment)).parse(input)
}

fn parse_statement_assignment(input: &str) -> IResult<&str, Statement>{
    let (input, var_expr) = eat_ws(parse_var)(input)?;
    let (input, rhs_expr) = eat_ws(parse_assignment)(input)?;
    Ok((input, Statement::Assign {
        name: var_expr,
        value: rhs_expr,
    }))
}

fn parse_var_declaration(input: &str) -> IResult<&str, Statement>{
    let (input, var_expr) = preceded(eat_ws(tag("let")), eat_ws(parse_var)).parse(input)?;
    let (input, maybe_assignment) = opt(eat_ws(parse_assignment)).parse(input)?;

    match maybe_assignment {
        Some(rhs_expr) => {
            Ok((input, Statement::DeclAssign {
                name: var_expr, 
                value: rhs_expr
            }))
        },
        None => {
            Ok((input, Statement::Decl(var_expr)))
        }
    }
}

fn parse_assignment(input: &str) -> IResult<&str, Expr>{
    preceded(eat_ws(tag("=")), eat_ws(parse_expr))(input)
}

// expression parsing

/*
    expr (additive_term) ::= multiplicative_term (("+"|"-") multiplicative_term)*
    multiplicative_term ::= unary_term (("*"|"/") unary_term)*
*/
fn parse_expr(input: &str) -> IResult<&str, Expr>{
    let (input, first_mul) = eat_ws(parse_mul_term)(input)?;
    let (input, additive_terms) = many0(pair(
        alt((eat_ws(tag("+")), eat_ws(tag("-")))),
        eat_ws(parse_mul_term))
    )(input)?;

    let mut current_tail: Expr = first_mul; // can be either Mul<Box<Expr>> (initially) or Add<Box<Expr>, Box<Expr>>

    for (operator, mul_term) in additive_terms {
        match operator {
            "+" => {
                current_tail = Expr::Add(Box::new(current_tail), Box::new(mul_term));
            },
            "-" => {
                current_tail = Expr::Sub(Box::new(current_tail), Box::new(mul_term));
            },
            _ => ()
        }
    }

    Ok((input, current_tail))
}

fn parse_mul_term(input: &str) -> IResult<&str, Expr>{
    eat_ws(parse_int_literal)(input)
}

/*
    unary_term ::= "-" unary_term | primary_term
*/

// fn parse_unary_term(input: &str) -> IResult<&str, Expr>{

// }

/*
    primary_term ::= "(" expr ")" 
    | VAR_NAME
    | INT_LIT
*/

// fn parse_primary_term(input: &str) -> IResult<&str, Expr>{

// }

fn parse_var(input: &str) -> IResult<&str, Expr>{
    let (input, identifier) = eat_ws(parse_identifier)(input)?;
    Ok((input, Expr::Var(identifier.to_string())))
}

// literals
fn parse_int_literal(input: &str) -> IResult<&str, Expr>{
    let (input, digits) = take_while1(nom::AsChar::is_dec_digit)(input)?;

    Ok((input, Expr::Int(i64::from_str(digits).unwrap())))
}

fn parse_identifier(input: &str) -> IResult<&str, &str>{
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