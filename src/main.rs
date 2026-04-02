use std::env;
use std::fs;

mod ast;
mod interpreter;
mod parser;
mod web;

use interpreter::run;
use parser::parse;

/*
    Entrypoint to the program: handles finding the file to interpret,
    converts it to a string,
    then hands it off to the parser.
*/
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut i =0;

    for arg in args.iter() {
        println!("{} {}",i,  arg);
        i += 1;
    }

    if args.len() < 2 {
        eprintln!(
            "Usage:\n  {} <filename>        Run a program file\n  {} --web [port]     Start the web playground (default port: 8080)",
            args[0], args[0]
        );
        return;
    }

    if args[1] == "--web" {
        let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8080);
        web::serve(port);
        return;
    }

    // CLI mode
    let filename = &args[1];
    match fs::read_to_string(filename) {
        Ok(text) => {
            let statements = parse(&text);
            run(&statements);
        }
        Err(e) => eprintln!("Error reading file '{}': {}", filename, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
