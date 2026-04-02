# 3YP
Third Year Project: Yet Another Probabilistic Programming Language (YAPPL). 

## Trying the Project

If you don't want to build and run the project, you should be able to play around with the language at 

## Locally Testing the Project

For targeting a specific file:
```
cargo run 
```

For running the web interface locally:
```
cargo run -- --web
```
And then visit `http://localhost:8080`.

## Building Instructions

You can either obtain an `exe` of the most recent release of the project from the GitHub page, or you can build your own with Rust and Cargo.

1) Install Rust and Cargo - guide here: https://doc.rust-lang.org/cargo/getting-started/installation.html

2) Run `cargo build --release`

3) Your executable should be found at the relative path `target/release/third-year-project` 

## Running Instructions

You should have an executable or program of some sort at this stage. If you are using an executable, test by running:

```
./path-to-exe Sample/Passing/MulAddOutput.txt
```

This should run the interpreter on a simple example of a syntactically and semantically correct YAPPL program.