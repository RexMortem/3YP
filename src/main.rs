use std::env;
use std::fs;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, multispace1, line_ending},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Statement<'a> {
    Assignment { name: &'a str, value: &'a str },
    FunctionCall { name: &'a str, arg: &'a str },
}

// ---------- Basic Parsers ----------

// Recognise variable/function identifiers
fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphabetic())(input)
}

// Recognise literals
fn value(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphanumeric())(input)
}

fn assignment(input: &str) -> IResult<&str, Statement> {
    map(
        separated_pair(
            terminated(identifier, multispace0),
            preceded(multispace0, char('=')),
            preceded(multispace0, value),
        ),
        |(name, value)| Statement::Assignment { name, value },
    )(input)
}

// For parsing calls to "output"
fn parse_function_call(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            identifier,
            delimited(
                preceded(multispace0, char('(')),
                preceded(multispace0, identifier),
                preceded(multispace0, char(')')),
            ),
        )),
        |(name, arg)| Statement::FunctionCall { name, arg },
    )(input)
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((assignment, parse_function_call))(input)
}

fn parse_program(input: &str) -> IResult<&str, Vec<Statement>> {
    separated_list0(
        delimited(
            multispace0,
            alt((
                map(char(';'), |_| "\n"), 
                line_ending,
            )),
            multispace0,
        ),
        parse_statement,
    )(input)
}

/*
    Run should be given a program (as a string) to execute
*/
fn run(input: &str){
    let parsedProgram = parse_program(input);
    
    match parsedProgram {
        Ok((_rest, statements)) => {
            println!("\n{:#?}", statements);
        }
        Err(e) => {
            eprintln!("{:#?}", e);
        }
    }
}

/*
    Entrypoint to the program: handles finding the file to interpret, 
    converts it to a string,
    then hands it off to the parser.
*/
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No filename given - usage is: ./{} <FileName>", args[0]);
        return;
    }

    let filename = &args[1];

    let contents = fs::read_to_string(filename);

    match contents {
        Ok(text) => {
            run(&text);
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            return;
        }
    }
}