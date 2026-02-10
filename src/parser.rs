use std::str::FromStr;

use nom::{
  IResult,
  Parser,
  sequence::{delimited, terminated, pair, preceded},
  character::complete::{multispace0, satisfy},
  bytes::complete::{tag, take_while, take_while1},
  multi::many0,
  branch::{alt},
  combinator::{opt, recognize, map}
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
    alt((parse_var_declaration, parse_statement_assignment, hardcoded_output)).parse(input)
}

/*
    HERE FOR TESTING - will probably change implementation later
*/
fn hardcoded_output(input: &str) -> IResult<&str, Statement>{
    let (input, expr_to_output) = delimited(
        eat_ws(tag("output(")),
        parse_expr,
        eat_ws(tag(")"))
    )(input)?;

    Ok((input, Statement::HardcodedOutput(expr_to_output)))
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
    preceded(eat_ws(tag("=")), eat_ws(parse_expr_head))(input)
}

// expression parsing

/*
    arg_list = arg ("," arg)* | epsilon
*/
fn parse_arg_list(input: &str) -> IResult<&str, Vec<Expr>>{
    let (input, first_expr) = eat_ws(parse_expr)(input)?;
    let (input, rest_args) = many0(
        preceded(
            eat_ws(tag(",")),
            eat_ws(parse_expr)
        )
    )(input)?;

    let mut args = vec![first_expr];
    args.extend(rest_args);
    Ok((input, args))
}

fn parse_arg_list_optional(input: &str) -> IResult<&str, Vec<Expr>>{
    opt(parse_arg_list)(input).map(|(input, maybe_args)| {
        (input, maybe_args.unwrap_or_default())
    })
}

/*
    expr ::= additive_term | dist_inst
*/
fn parse_expr_head(input: &str) -> IResult<&str, Expr>{
    parse_expr(input)
}


/*
    expr (additive_term) ::= multiplicative_term (("+"|"-") multiplicative_term)*
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

/*
    multiplicative_term ::= unary_term (("*"|"/") unary_term)*
*/
fn parse_mul_term(input: &str) -> IResult<&str, Expr>{
    let (input, first_unary) = eat_ws(parse_unary_term)(input)?;
    let (input, mul_terms) = many0(pair(
        alt((eat_ws(tag("*")), eat_ws(tag("/")))),
        eat_ws(parse_unary_term))
    )(input)?;

    let mut current_tail: Expr = first_unary; 

    for (operator, unary_term) in mul_terms {
        match operator {
            "*" => {
                current_tail = Expr::Mul(Box::new(current_tail), Box::new(unary_term));
            },
            "/" => {
                current_tail = Expr::Div(Box::new(current_tail), Box::new(unary_term));
            },
            _ => ()
        }
    }

    Ok((input, current_tail))
}

/*
    unary_term ::= "-" unary_term | primary_term
*/

fn parse_unary_term(input: &str) -> IResult<&str, Expr>{
    alt((
        map(
            preceded(eat_ws(tag("-")), parse_unary_term),
            |expr| Expr::Neg(Box::new(expr))
        ),
        parse_primary_term
    )).parse(input)
}

/*
    primary_term ::= "(" expr ")" 
    | VAR_NAME
    | INT_LIT
*/

fn parse_primary_term(input: &str) -> IResult<&str, Expr>{
    alt((
        delimited(
            eat_ws(tag("(")),
            parse_expr,
            eat_ws(tag(")"))
        ),
        parse_func_call,
        parse_var,
        parse_int_literal
    )).parse(input)
}

fn parse_func_call(input: &str) -> IResult<&str, Expr>{
    let (input, func_name) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag("("))(input)?;
    let (input, args) = parse_arg_list_optional(input)?;
    let (input, _) = eat_ws(tag(")"))(input)?;

    // Check if this is a known distribution
    match func_name {
        "uniform" => {
            if args.len() != 2 {
                return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Count)));
            }
            match (&args[0], &args[1]) {
                (Expr::Int(a), Expr::Int(b)) => Ok((input, Expr::Dist(Dist::Uniform(*a, *b)))),
                _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))),
            }
        },
        _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    }
}

fn parse_var(input: &str) -> IResult<&str, Expr>{
    let (input, identifier) = eat_ws(parse_identifier)(input)?;
    let (input, opt_method_call) = opt(parse_method_call)(input)?;

    match opt_method_call {
        Some((method_name, args)) => {
            Ok((input, Expr::DistMethodCall {
                var: identifier.to_string(),
                method: method_name,
                args,
            }))
        },
        None => {
            Ok((input, Expr::Var(identifier.to_string())))
        }
    }
}

fn parse_method_call(input: &str) -> IResult<&str, (String, Vec<Expr>)>{
    let (input, _) = eat_ws(tag(":"))(input)?;
    let (input, method_name) = eat_ws(parse_identifier)(input)?;
    let (input, args) = delimited(
        eat_ws(tag("(")),
        parse_arg_list_optional,
        eat_ws(tag(")"))
    )(input)?;

    Ok((input, (method_name.to_string(), args)))
}

// literals
fn parse_int_literal(input: &str) -> IResult<&str, Expr>{
    let (input, digits) = take_while1(nom::AsChar::is_dec_digit)(input)?;

    Ok((input, Expr::Int(i64::from_str(digits).unwrap())))
}

fn parse_identifier(input: &str) -> IResult<&str, &str>{
    recognize(pair(
        satisfy(nom::AsChar::is_alpha),
        take_while(nom::AsChar::is_alphanum)
    ))(input)
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