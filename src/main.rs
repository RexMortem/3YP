use std::env;
use std::fs;

mod parser;
use parser::parse;

mod ast;

mod interpreter;
use interpreter::run;

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
            let statements = parse(&text);
            run(&statements)
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
