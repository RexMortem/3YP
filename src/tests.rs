/// Unit test suite for the YAPPL interpreter.
///
/// Tests are driven by sidecar `.expected` files alongside each `.txt` program:
///   - `Sample/Deterministic/Passing/Foo.txt`  →  exact stdout match against `Foo.expected`
///   - `Sample/Deterministic/Failing/Bar.txt`  →  program must error; error message must
///     contain the substring in `Bar.expected`
///
/// Run with `cargo test` or with the `--test` CLI flag.
use std::path::Path;

use crate::interpreter::try_run_program;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Run a program that should succeed and compare output against expected.
pub fn run_passing_test(source_path: &str) {
    let expected_path = source_path.replace(".txt", ".expected");

    let source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|_| panic!("Cannot read source file: {}", source_path));
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Cannot read expected file: {}", expected_path));

    match try_run_program(&source) {
        Ok(output) => {
            let actual = output.trim().to_string();
            let expected = expected.trim().to_string();
            assert_eq!(
                actual, expected,
                "\nTest FAILED (wrong output): {}\n  expected: {:?}\n  got:      {:?}",
                source_path, expected, actual
            );
        }
        Err(msg) => {
            panic!(
                "\nTest FAILED (unexpected error): {}\n  error: {}",
                source_path, msg
            );
        }
    }
}

/// Run a program that should fail and check the error message contains the expected substring.
pub fn run_failing_test(source_path: &str) {
    let expected_path = source_path.replace(".txt", ".expected");

    let source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|_| panic!("Cannot read source file: {}", source_path));
    let expected_substr = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Cannot read expected file: {}", expected_path));
    let expected_substr = expected_substr.trim();

    match try_run_program(&source) {
        Ok(output) => {
            panic!(
                "\nTest FAILED (should have errored): {}\n  got output: {:?}",
                source_path, output
            );
        }
        Err(msg) => {
            assert!(
                msg.contains(expected_substr),
                "\nTest FAILED (wrong error message): {}\n  expected to contain: {:?}\n  got: {:?}",
                source_path, expected_substr, msg
            );
        }
    }
}

// ── Passing tests ─────────────────────────────────────────────────────────────

macro_rules! passing_test {
    ($test_name:ident, $file:expr) => {
        #[test]
        fn $test_name() {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("Sample/Deterministic/Passing")
                .join($file);
            run_passing_test(path.to_str().unwrap());
        }
    };
}

passing_test!(basic_add_output,          "BasicAddOutput.txt");
passing_test!(basic_discrete_dist,       "BasicDiscreteDist.txt");
passing_test!(basic_discrete_dist_ops,   "BasicDiscreteDistOps.txt");
passing_test!(basic_dist_use,            "BasicDistUse.txt");
passing_test!(basic_dist_use2,           "BasicDistUse2.txt");
passing_test!(basic_dist_use2_exprs,     "BasicDistUse2Exprs.txt");
passing_test!(basic_dist_use2_edge1,     "BasicDistUse2edgeCase1.txt");
passing_test!(basic_dist_use2_edge2,     "BasicDistUse2edgeCase2.txt");
passing_test!(basic_uniform_continuous,  "BasicUniformContinuous.txt");
passing_test!(combining_dists,           "CombiningDists.txt");
passing_test!(complex_add_output,        "ComplexAddOutput.txt");
passing_test!(mul_add_output,            "MulAddOutput.txt");
passing_test!(two_dice_roll,             "TwoDiceRoll.txt");
passing_test!(uniform_continuous_floats, "UniformContinuousFloatArgs.txt");
passing_test!(comments,                  "Comments.txt");
passing_test!(boolean_expressions,       "BooleanExpressions.txt");
passing_test!(if_else,                   "IfElse.txt");
passing_test!(functions,                 "Functions.txt");
passing_test!(factorial,                 "Factorial.txt");
passing_test!(modulo,                    "Modulo.txt");
passing_test!(array_literal,             "ArrayLiteral.txt");
passing_test!(map_deterministic,         "MapDeterministic.txt");
passing_test!(distribution_equality,     "DistributionEquality.txt");
passing_test!(markov_chain,              "MarkovChain.txt");

// ── Failing tests ─────────────────────────────────────────────────────────────

macro_rules! failing_test {
    ($test_name:ident, $file:expr) => {
        #[test]
        fn $test_name() {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("Sample/Deterministic/Failing")
                .join($file);
            run_failing_test(path.to_str().unwrap());
        }
    };
}

failing_test!(failed_add,                   "FailedAdd.txt");
failing_test!(no_semicolons,                "NoSemicolons.txt");
failing_test!(missing_semicolon,            "MissingSemicolon.txt");
failing_test!(bad_expression,               "BadExpression.txt");
failing_test!(undefined_variable,           "UndefinedVariable.txt");
failing_test!(type_mismatch,                "TypeMismatch.txt");
failing_test!(wrong_arg_count,              "WrongArgCount.txt");
failing_test!(unknown_function,             "UnknownFunction.txt");
failing_test!(division_by_zero,             "DivisionByZero.txt");
failing_test!(pb_function_without_conf,     "PbFunctionWithoutConfidence.txt");
