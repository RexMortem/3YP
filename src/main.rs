use std::env;
use std::fs;
use std::path::Path;

mod ast;
mod interpreter;
mod parser;
mod visualiser;
mod web;

#[cfg(test)]
mod tests;

use interpreter::{run, try_run_program};
use parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage:\n  {} <filename>      Run a program file\n  {} --web [port]   Start the web playground (default port: 8080)\n  {} --test         Run the deterministic test suite",
            args[0], args[0], args[0]
        );
        return;
    }

    match args[1].as_str() {
        "--web" => {
            let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8080);
            web::serve(port);
        }
        "--test" => {
            run_test_suite();
        }
        filename => {
            match fs::read_to_string(filename) {
                Ok(text) => {
                    let items = parse(&text);
                    run(&items);
                }
                Err(e) => eprintln!("Error reading file '{}': {}", filename, e),
            }
        }
    }
}

// ── CLI Test Runner ───────────────────────────────────────────────────────────

fn run_test_suite() {
    // Silence the default panic hook so error-test panics don't spam stderr.
    std::panic::set_hook(Box::new(|_| {}));

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("Sample/Deterministic");
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<String> = Vec::new();

    println!("\nRunning YAPPL deterministic test suite\n");

    for (dir, expect_failure) in [("Passing", false), ("Failing", true)] {
        let dir_path = root.join(dir);
        let mut entries: Vec<_> = fs::read_dir(&dir_path)
            .unwrap_or_else(|_| panic!("Cannot read directory: {:?}", dir_path))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "txt").unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.path());

        println!("  [{}]", dir);
        for entry in entries {
            let src_path = entry.path();
            let exp_path = src_path.with_extension("expected");

            let name = src_path.file_stem().unwrap().to_string_lossy().to_string();

            if !exp_path.exists() {
                println!("    SKIP  {} (no .expected file)", name);
                continue;
            }

            let source = fs::read_to_string(&src_path).unwrap();
            let expected = fs::read_to_string(&exp_path).unwrap();
            let expected = expected.trim();

            let result = try_run_program(&source);
            let (ok, detail) = if expect_failure {
                match result {
                    Err(msg) if msg.contains(expected) => (true, String::new()),
                    Err(msg) => (
                        false,
                        format!("expected error containing {:?}, got: {:?}", expected, msg),
                    ),
                    Ok(out) => (
                        false,
                        format!("expected failure but got output: {:?}", out.trim()),
                    ),
                }
            } else {
                match result {
                    Ok(output) if output.trim() == expected => (true, String::new()),
                    Ok(output) => (
                        false,
                        format!("expected {:?}, got {:?}", expected, output.trim()),
                    ),
                    Err(msg) => (false, format!("unexpected error: {}", msg)),
                }
            };

            if ok {
                println!("    PASS  {}", name);
                passed += 1;
            } else {
                println!("    FAIL  {}  —  {}", name, detail);
                failed += 1;
                failures.push(name);
            }
        }
        println!();
    }

    println!("Results: {} passed, {} failed\n", passed, failed);
    if !failures.is_empty() {
        eprintln!("Failed tests: {}", failures.join(", "));
        std::process::exit(1);
    }
}
