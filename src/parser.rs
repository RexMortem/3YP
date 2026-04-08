use std::str::FromStr;

use nom::{
    IResult,
    Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{multispace0, satisfy},
    combinator::{map, opt, peek, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
};

use crate::ast::*;

// ── Whitespace & Comment Handling ────────────────────────────────────────────

/// Skips whitespace and `// line comments` in a loop.
fn ws0(input: &str) -> IResult<&str, ()> {
    let mut i = input;
    loop {
        let (j, _) = multispace0(i)?;
        i = j;
        if i.starts_with("//") {
            let end = i.find('\n').map(|n| n + 1).unwrap_or(i.len());
            i = &i[end..];
        } else {
            break;
        }
    }
    Ok((i, ()))
}

fn eat_ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(ws0, inner, ws0)
}

// ── Keyword Helper ───────────────────────────────────────────────────────────

/// Matches `kw` only when NOT followed by an alphanumeric character or `_`,
/// ensuring we don't accidentally match a prefix of an identifier like `mod_exp`.
fn keyword<'a>(kw: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    move |input| {
        let (rest, matched) = tag(kw)(input)?;
        match rest.chars().next() {
            Some(c) if c.is_alphanumeric() || c == '_' => Err(nom::Err::Error(
                nom::error::Error::new(input, nom::error::ErrorKind::Tag),
            )),
            _ => Ok((rest, matched)),
        }
    }
}

// ── Identifiers ──────────────────────────────────────────────────────────────

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        satisfy(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))(input)
}

// ── Top-Level ────────────────────────────────────────────────────────────────

fn parse_program(input: &str) -> IResult<&str, Vec<ProgramItem>> {
    terminated(many0(eat_ws(parse_program_item)), ws0)(input)
}

fn parse_program_item(input: &str) -> IResult<&str, ProgramItem> {
    alt((
        map(parse_enum_def, ProgramItem::EnumDef),  // enum before fn to avoid ambiguity
        map(parse_fn_def, ProgramItem::FuncDef),
        map(parse_pb_func_def, ProgramItem::PbFuncDef),
        map(terminated(parse_statement, eat_ws(tag(";"))), ProgramItem::Statement),
    ))
    .parse(input)
}

// ── Statements ───────────────────────────────────────────────────────────────

fn parse_statement_list(input: &str) -> IResult<&str, Vec<Statement>> {
    many0(terminated(parse_statement, eat_ws(tag(";"))))(input)
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_pb_call_assign,      // must precede parse_map_call_assign and parse_var_declaration
        parse_distribution_of,     // must precede parse_map_call_assign (both start with `let`)
        parse_map_call_assign,     // must precede parse_var_declaration (both start with `let`)
        parse_return_stmt,
        parse_if_stmt,
        parse_var_declaration,
        hardcoded_output,
        parse_statement_assignment,
    ))
    .parse(input)
}

fn parse_block(input: &str) -> IResult<&str, Vec<Statement>> {
    delimited(eat_ws(tag("{")), parse_statement_list, eat_ws(tag("}")))(input)
}

fn hardcoded_output(input: &str) -> IResult<&str, Statement> {
    let (input, expr) = delimited(eat_ws(tag("output(")), parse_expr, eat_ws(tag(")")))(input)?;
    Ok((input, Statement::HardcodedOutput(expr)))
}

fn parse_return_stmt(input: &str) -> IResult<&str, Statement> {
    let (input, _) = eat_ws(keyword("return"))(input)?;
    // The expression is optional (bare `return;`)
    let (input, expr) = opt(eat_ws(parse_expr))(input)?;
    Ok((input, Statement::Return(expr)))
}

fn parse_if_stmt(input: &str) -> IResult<&str, Statement> {
    let (input, _) = eat_ws(keyword("if"))(input)?;
    let (input, cond) = eat_ws(parse_expr)(input)?;
    let (input, then_block) = parse_block(input)?;
    let (input, else_block) = opt(preceded(eat_ws(keyword("else")), parse_block))(input)?;
    Ok((input, Statement::If { cond, then_block, else_block }))
}

fn parse_statement_assignment(input: &str) -> IResult<&str, Statement> {
    let (input, var_expr) = eat_ws(parse_var)(input)?;
    let (input, rhs) = eat_ws(parse_assignment_rhs)(input)?;
    Ok((input, Statement::Assign { name: var_expr, value: rhs }))
}

fn parse_var_declaration(input: &str) -> IResult<&str, Statement> {
    let (input, _) = eat_ws(keyword("let"))(input)?;
    let (input, var_expr) = eat_ws(parse_var)(input)?;
    let (input, maybe_rhs) = opt(eat_ws(parse_assignment_rhs))(input)?;
    match maybe_rhs {
        Some(rhs) => Ok((input, Statement::DeclAssign { name: var_expr, value: rhs })),
        None => Ok((input, Statement::Decl(var_expr))),
    }
}

/// `let result_var, info_var = func_name(args) with confidence >= <float>`
fn parse_pb_call_assign(input: &str) -> IResult<&str, Statement> {
    let (input, _) = eat_ws(keyword("let"))(input)?;
    let (input, result_var) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag(","))(input)?;
    let (input, info_var) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag("="))(input)?;
    let (input, func_name) = eat_ws(parse_identifier)(input)?;
    let (input, args) =
        delimited(eat_ws(tag("(")), parse_arg_list_optional, eat_ws(tag(")")))(input)?;
    let (input, _) = eat_ws(keyword("with"))(input)?;
    let (input, _) = eat_ws(keyword("confidence"))(input)?;
    let (input, _) = eat_ws(tag(">="))(input)?;
    let (input, confidence) = eat_ws(parse_float_value)(input)?;
    Ok((input, Statement::PbCallAssign {
        result_var: result_var.to_string(),
        info_var: info_var.to_string(),
        func_name: func_name.to_string(),
        args,
        confidence,
    }))
}

/// `let var = map(func_name, array_expr) [with confidence >= float]`
fn parse_map_call_assign(input: &str) -> IResult<&str, Statement> {
    let (input, _) = eat_ws(keyword("let"))(input)?;
    let (input, var) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag("="))(input)?;
    let (input, _) = eat_ws(keyword("map"))(input)?;
    let (input, _) = eat_ws(tag("("))(input)?;
    let (input, func_name) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag(","))(input)?;
    let (input, array_expr) = eat_ws(parse_expr)(input)?;
    let (input, _) = eat_ws(tag(")"))(input)?;
    let (input, confidence) = opt(parse_with_confidence)(input)?;
    Ok((input, Statement::MapCallAssign {
        var: var.to_string(),
        func_name: func_name.to_string(),
        array_expr,
        confidence,
    }))
}

fn parse_with_confidence(input: &str) -> IResult<&str, f64> {
    let (input, _) = eat_ws(keyword("with"))(input)?;
    let (input, _) = eat_ws(keyword("confidence"))(input)?;
    let (input, _) = eat_ws(tag(">="))(input)?;
    let (input, val) = eat_ws(parse_float_value)(input)?;
    Ok((input, val))
}

/// Parse a plain non-negative integer literal as `i64`.
fn parse_i64(input: &str) -> IResult<&str, i64> {
    let (input, s) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let n: i64 = s.parse().unwrap_or(0);
    Ok((input, n))
}

/// `let var = distribution_of(func_name(args), analytical|empirical[, N]|bayesian[, N])`
fn parse_distribution_of(input: &str) -> IResult<&str, Statement> {
    let (input, _) = eat_ws(keyword("let"))(input)?;
    let (input, var) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag("="))(input)?;
    let (input, _) = eat_ws(keyword("distribution_of"))(input)?;
    let (input, _) = eat_ws(tag("("))(input)?;
    // Parse func_name(args)
    let (input, func_name) = eat_ws(parse_identifier)(input)?;
    let (input, args) =
        delimited(eat_ws(tag("(")), parse_arg_list_optional, eat_ws(tag(")")))(input)?;
    let (input, _) = eat_ws(tag(","))(input)?;
    // Parse mode keyword
    let (input, mode_str) = eat_ws(alt((
        keyword("analytical"),
        keyword("empirical"),
        keyword("bayesian"),
    )))(input)?;
    // Optional ", N" for empirical and bayesian (default 100)
    let (input, n_opt) = if mode_str != "analytical" {
        opt(preceded(eat_ws(tag(",")), eat_ws(parse_i64)))(input)?
    } else {
        (input, None)
    };
    let (input, _) = eat_ws(tag(")"))(input)?;
    let default_n = 100i64;
    let mode = match mode_str {
        "analytical" => DistributionOfMode::Analytical,
        "empirical"  => DistributionOfMode::Empirical(n_opt.unwrap_or(default_n)),
        "bayesian"   => DistributionOfMode::Bayesian(n_opt.unwrap_or(default_n)),
        _            => unreachable!(),
    };
    Ok((input, Statement::DistributionOf {
        var: var.to_string(),
        func_name: func_name.to_string(),
        args,
        mode,
    }))
}

fn parse_assignment_rhs(input: &str) -> IResult<&str, Expr> {
    preceded(eat_ws(tag("=")), eat_ws(parse_expr))(input)
}

// ── Function Definitions ──────────────────────────────────────────────────────

fn parse_fn_def(input: &str) -> IResult<&str, FuncDef> {
    let (input, _) = eat_ws(keyword("fn"))(input)?;
    let (input, name) = eat_ws(parse_identifier)(input)?;
    let (input, params) =
        delimited(eat_ws(tag("(")), parse_param_list, eat_ws(tag(")")))(input)?;
    let (input, _) = eat_ws(tag("->"))(input)?;
    let (input, return_type) = eat_ws(parse_type)(input)?;
    let (input, body) = parse_block(input)?;
    Ok((input, FuncDef { name: name.to_string(), params, return_type, body }))
}

fn parse_pb_func_def(input: &str) -> IResult<&str, PbFuncDef> {
    let (input, _) = eat_ws(keyword("pb"))(input)?;
    let (input, _) = eat_ws(keyword("function"))(input)?;
    let (input, name) = eat_ws(parse_identifier)(input)?;
    let (input, params) =
        delimited(eat_ws(tag("(")), parse_param_list, eat_ws(tag(")")))(input)?;
    let (input, _) = eat_ws(tag("->"))(input)?;
    let (input, return_type) = eat_ws(parse_type)(input)?;
    let (input, (error_class, error_distribution)) =
        delimited(eat_ws(tag("{")), parse_pb_metadata, eat_ws(tag("}")))(input)?;
    let (input, body) = parse_block(input)?;
    Ok((input, PbFuncDef {
        name: name.to_string(),
        params,
        return_type,
        error_class,
        error_distribution,
        body,
    }))
}

fn parse_pb_metadata(input: &str) -> IResult<&str, (ErrorClass, String)> {
    let (input, _) = eat_ws(tag("error_class"))(input)?;
    let (input, _) = eat_ws(tag(":"))(input)?;
    let (input, ec) = eat_ws(parse_error_class)(input)?;
    let (input, _) = eat_ws(tag(","))(input)?;
    let (input, _) = eat_ws(tag("error_distribution"))(input)?;
    let (input, _) = eat_ws(tag(":"))(input)?;
    let (input, dist_name) = eat_ws(parse_identifier)(input)?;
    Ok((input, (ec, dist_name.to_string())))
}

fn parse_error_class(input: &str) -> IResult<&str, ErrorClass> {
    alt((
        map(keyword("RP"), |_| ErrorClass::RP),
        map(keyword("coRP"), |_| ErrorClass::CoRP),
        map(keyword("BPP"), |_| ErrorClass::BPP),
    ))
    .parse(input)
}

fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        map(keyword("int"), |_| Type::Int),
        map(keyword("float"), |_| Type::Float),
        map(keyword("bool"), |_| Type::Bool),
        parse_type_dist_of,
        parse_type_named,
    ))
    .parse(input)
}

/// Parses `Discrete<TypeName>` as `Type::DistOf("TypeName")`.
fn parse_type_dist_of(input: &str) -> IResult<&str, Type> {
    let (input, _) = eat_ws(tag("Discrete"))(input)?;
    let (input, _) = eat_ws(tag("<"))(input)?;
    let (input, type_name) = eat_ws(parse_identifier)(input)?;
    if !type_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input, nom::error::ErrorKind::Tag,
        )));
    }
    let (input, _) = eat_ws(tag(">"))(input)?;
    Ok((input, Type::DistOf(type_name.to_string())))
}

/// Parses an uppercase identifier as a named enum type, e.g. `Weather` → `Type::Named("Weather")`.
fn parse_type_named(input: &str) -> IResult<&str, Type> {
    let (rest, name) = eat_ws(parse_identifier)(input)?;
    if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input, nom::error::ErrorKind::Tag,
        )));
    }
    Ok((rest, Type::Named(name.to_string())))
}

// ── Enum Definitions ──────────────────────────────────────────────────────────

/// Parses `enum TypeName { Variant1, Variant2, ... }`
/// Both the type name and every variant must start with an uppercase letter.
fn parse_enum_def(input: &str) -> IResult<&str, EnumDef> {
    let (input, _) = eat_ws(keyword("enum"))(input)?;
    let (input, name) = eat_ws(parse_identifier)(input)?;
    if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input, nom::error::ErrorKind::Tag,
        )));
    }
    let (input, variants) =
        delimited(eat_ws(tag("{")), parse_variant_list, eat_ws(tag("}")))(input)?;
    Ok((input, EnumDef { name: name.to_string(), variants }))
}

fn parse_variant_list(input: &str) -> IResult<&str, Vec<String>> {
    let (input, first) = eat_ws(parse_uppercase_identifier)(input)?;
    let (input, rest) =
        many0(preceded(eat_ws(tag(",")), eat_ws(parse_uppercase_identifier)))(input)?;
    let mut variants = vec![first.to_string()];
    variants.extend(rest.into_iter().map(|s| s.to_string()));
    Ok((input, variants))
}

/// Like `parse_identifier` but fails if the first character is not uppercase.
fn parse_uppercase_identifier(input: &str) -> IResult<&str, &str> {
    let (rest, ident) = parse_identifier(input)?;
    if !ident.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input, nom::error::ErrorKind::Tag,
        )));
    }
    Ok((rest, ident))
}

fn parse_param_list(input: &str) -> IResult<&str, Vec<FuncParam>> {
    let (input, maybe) = opt(pair(
        eat_ws(parse_param),
        many0(preceded(eat_ws(tag(",")), eat_ws(parse_param))),
    ))(input)?;
    let params = match maybe {
        Some((first, rest)) => {
            let mut v = vec![first];
            v.extend(rest);
            v
        }
        None => vec![],
    };
    Ok((input, params))
}

fn parse_param(input: &str) -> IResult<&str, FuncParam> {
    let (input, name) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag(":"))(input)?;
    let (input, ty) = eat_ws(parse_type)(input)?;
    Ok((input, FuncParam { name: name.to_string(), ty }))
}

// ── Argument / Pair Lists ─────────────────────────────────────────────────────

fn parse_arg_list_optional(input: &str) -> IResult<&str, Vec<Expr>> {
    opt(parse_arg_list)(input).map(|(input, maybe)| (input, maybe.unwrap_or_default()))
}

fn parse_arg_list(input: &str) -> IResult<&str, Vec<Expr>> {
    let (input, first) = eat_ws(parse_expr)(input)?;
    let (input, rest) =
        many0(preceded(eat_ws(tag(",")), eat_ws(parse_expr)))(input)?;
    let mut args = vec![first];
    args.extend(rest);
    Ok((input, args))
}

fn parse_discrete_pair_list_optional(
    input: &str,
) -> IResult<&str, Vec<(Box<Expr>, Box<Expr>)>> {
    opt(parse_discrete_pair_list)(input)
        .map(|(input, maybe)| (input, maybe.unwrap_or_default()))
}

fn parse_discrete_pair_list(input: &str) -> IResult<&str, Vec<(Box<Expr>, Box<Expr>)>> {
    let (input, first) = eat_ws(parse_discrete_pair)(input)?;
    let (input, rest) =
        many0(preceded(eat_ws(tag(",")), eat_ws(parse_discrete_pair)))(input)?;
    let mut pairs = vec![first];
    pairs.extend(rest);
    Ok((input, pairs))
}

fn parse_discrete_pair(input: &str) -> IResult<&str, (Box<Expr>, Box<Expr>)> {
    let (input, key) = eat_ws(parse_expr)(input)?;
    let (input, _) = eat_ws(tag(":"))(input)?;
    let (input, value) = eat_ws(parse_expr)(input)?;
    Ok((input, (Box::new(key), Box::new(value))))
}

// ── Expression Parsing ────────────────────────────────────────────────────────
// Precedence (lowest → highest):
//   OR → AND → CMP → ADD → MUL → UNARY → PRIMARY(+postfix)

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    parse_or_expr(input)
}

fn parse_or_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = eat_ws(parse_and_expr)(input)?;
    let (input, rest) =
        many0(preceded(eat_ws(tag("||")), eat_ws(parse_and_expr)))(input)?;
    let mut result = first;
    for rhs in rest {
        result = Expr::Or(Box::new(result), Box::new(rhs));
    }
    Ok((input, result))
}

fn parse_and_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = eat_ws(parse_cmp_expr)(input)?;
    let (input, rest) =
        many0(preceded(eat_ws(tag("&&")), eat_ws(parse_cmp_expr)))(input)?;
    let mut result = first;
    for rhs in rest {
        result = Expr::And(Box::new(result), Box::new(rhs));
    }
    Ok((input, result))
}

fn parse_cmp_expr(input: &str) -> IResult<&str, Expr> {
    let (input, lhs) = eat_ws(parse_add_expr)(input)?;

    // Try `~=` (approximate equality) before regular operators.
    // Syntax: lhs ~= rhs (within tolerance)?
    if let Ok((rest, _)) = eat_ws(tag::<&str, &str, nom::error::Error<&str>>("~="))(input) {
        let (rest, rhs) = eat_ws(parse_add_expr)(rest)?;
        let (rest, tolerance) = opt(preceded(
            eat_ws(keyword("within")),
            eat_ws(parse_add_expr),
        ))(rest)?;
        return Ok((rest, Expr::ApproxEq(Box::new(lhs), Box::new(rhs), tolerance.map(Box::new))));
    }

    // Multi-char operators must be tried before single-char prefixes.
    let (input, maybe_op) = opt(pair(
        eat_ws(alt((
            tag("=="),
            tag("!="),
            tag("<="),
            tag(">="),
            tag("<"),
            tag(">"),
        ))),
        eat_ws(parse_add_expr),
    ))(input)?;
    match maybe_op {
        Some((op, rhs)) => {
            let expr = match op {
                "==" => Expr::Eq(Box::new(lhs), Box::new(rhs)),
                "!=" => Expr::Neq(Box::new(lhs), Box::new(rhs)),
                "<=" => Expr::Lte(Box::new(lhs), Box::new(rhs)),
                ">=" => Expr::Gte(Box::new(lhs), Box::new(rhs)),
                "<" => Expr::Lt(Box::new(lhs), Box::new(rhs)),
                ">" => Expr::Gt(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            };
            Ok((input, expr))
        }
        None => Ok((input, lhs)),
    }
}

fn parse_add_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = eat_ws(parse_mul_term)(input)?;
    let (input, rest) = many0(pair(
        eat_ws(alt((tag("+"), tag("-")))),
        eat_ws(parse_mul_term),
    ))(input)?;
    let mut result = first;
    for (op, rhs) in rest {
        result = match op {
            "+" => Expr::Add(Box::new(result), Box::new(rhs)),
            "-" => Expr::Sub(Box::new(result), Box::new(rhs)),
            _ => unreachable!(),
        };
    }
    Ok((input, result))
}

fn parse_mul_term(input: &str) -> IResult<&str, Expr> {
    let (input, first) = eat_ws(parse_unary_term)(input)?;
    // `mod` is tried as a keyword (fails if followed by `_` or alphanumeric)
    // so `mod_exp` is never consumed here.
    let (input, rest) = many0(pair(
        eat_ws(alt((tag("*"), tag("/"), tag("%"), keyword("mod")))),
        eat_ws(parse_unary_term),
    ))(input)?;
    let mut result = first;
    for (op, rhs) in rest {
        result = match op {
            "*" => Expr::Mul(Box::new(result), Box::new(rhs)),
            "/" => Expr::Div(Box::new(result), Box::new(rhs)),
            "%" | "mod" => Expr::Mod(Box::new(result), Box::new(rhs)),
            _ => unreachable!(),
        };
    }
    Ok((input, result))
}

fn parse_unary_term(input: &str) -> IResult<&str, Expr> {
    alt((
        map(preceded(eat_ws(tag("-")), parse_unary_term), |e| Expr::Neg(Box::new(e))),
        map(preceded(eat_ws(tag("!")), parse_unary_term), |e| Expr::Not(Box::new(e))),
        parse_primary_with_postfix,
    ))
    .parse(input)
}

/// After a primary expression, optionally consume chained `.method(args)` or `:method(args)` calls.
fn parse_primary_with_postfix(input: &str) -> IResult<&str, Expr> {
    let (input, base) = parse_primary_term(input)?;
    let (input, methods) = many0(alt((parse_dot_method, parse_colon_method)))(input)?;
    let mut result = base;
    for (method, args) in methods {
        result = Expr::ExprMethodCall { expr: Box::new(result), method, args };
    }
    Ok((input, result))
}

fn parse_dot_method(input: &str) -> IResult<&str, (String, Vec<Expr>)> {
    let (input, _) = eat_ws(tag("."))(input)?;
    let (input, method_name) = eat_ws(parse_identifier)(input)?;
    let (input, args) =
        delimited(eat_ws(tag("(")), parse_arg_list_optional, eat_ws(tag(")")))(input)?;
    Ok((input, (method_name.to_string(), args)))
}

fn parse_primary_term(input: &str) -> IResult<&str, Expr> {
    alt((
        delimited(eat_ws(tag("(")), parse_expr, eat_ws(tag(")"))),
        // Array literals: [expr, expr, ...]
        parse_array_literal,
        // Boolean literals must come before generic identifier/call parsing.
        map(eat_ws(keyword("true")), |_| Expr::Bool(true)),
        map(eat_ws(keyword("false")), |_| Expr::Bool(false)),
        // Function/constructor calls (includes Certain, Uncertain, distribution ctors).
        parse_func_call,
        // Variable reference (with optional legacy `:method()` suffix).
        parse_var,
        // Numeric literals.
        parse_number_literal,
    ))
    .parse(input)
}

fn parse_array_literal(input: &str) -> IResult<&str, Expr> {
    let (input, _) = eat_ws(tag("["))(input)?;
    let (input, elements) = parse_arg_list_optional(input)?;
    let (input, _) = eat_ws(tag("]"))(input)?;
    Ok((input, Expr::Array(elements)))
}

/// Parses any call of the form `name(args)`.
/// Peeks ahead so that if no `(` follows the identifier, parsing backtracks cleanly.
fn parse_func_call(input: &str) -> IResult<&str, Expr> {
    // Guard: only proceed if we see `identifier(` without consuming.
    let _ = peek(pair(eat_ws(parse_identifier), eat_ws(tag("("))))(input)?;

    let (input, func_name) = eat_ws(parse_identifier)(input)?;
    let (input, _) = eat_ws(tag("("))(input)?;

    match func_name {
        // ── Certainty markers ──────────────────────────────────────────────
        "Certain" => {
            let (input, inner) = parse_expr(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            Ok((input, Expr::Certain(Box::new(inner))))
        }
        "Uncertain" => {
            let (input, inner) = parse_expr(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            Ok((input, Expr::Uncertain(Box::new(inner))))
        }
        // ── Distribution constructors ──────────────────────────────────────
        "uniform" => {
            let (input, args) = parse_arg_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            if args.len() != 2 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Count,
                )));
            }
            Ok((
                input,
                Expr::Dist(Dist::Uniform(Box::new(args[0].clone()), Box::new(args[1].clone()))),
            ))
        }
        "uniformContinuous" => {
            let (input, args) = parse_arg_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            if args.len() != 2 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Count,
                )));
            }
            Ok((
                input,
                Expr::Dist(Dist::UniformContinuous(
                    Box::new(args[0].clone()),
                    Box::new(args[1].clone()),
                )),
            ))
        }
        "Discrete" => {
            let (input, pairs) = parse_discrete_pair_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            if pairs.is_empty() {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Count,
                )));
            }
            Ok((input, Expr::Dist(Dist::Discrete(pairs))))
        }
        "Bernoulli" => {
            let (input, args) = parse_arg_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            if args.len() != 1 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Count,
                )));
            }
            Ok((input, Expr::Dist(Dist::Bernoulli(Box::new(args[0].clone())))))
        }
        "Binomial" => {
            let (input, args) = parse_arg_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            if args.len() != 2 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Count,
                )));
            }
            Ok((
                input,
                Expr::Dist(Dist::Binomial(Box::new(args[0].clone()), Box::new(args[1].clone()))),
            ))
        }
        "Geometric" => {
            let (input, args) = parse_arg_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            if args.len() != 1 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Count,
                )));
            }
            Ok((input, Expr::Dist(Dist::Geometric(Box::new(args[0].clone())))))
        }
        // ── Generic function call ──────────────────────────────────────────
        _ => {
            let (input, args) = parse_arg_list_optional(input)?;
            let (input, _) = eat_ws(tag(")"))(input)?;
            Ok((input, Expr::FuncCall(func_name.to_string(), args)))
        }
    }
}

/// Variable reference. Colon and dot method calls are handled by `parse_primary_with_postfix`.
fn parse_var(input: &str) -> IResult<&str, Expr> {
    let (input, identifier) = eat_ws(parse_identifier)(input)?;
    // Block reserved keywords from appearing as variable names.
    if is_reserved_keyword(identifier) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    Ok((input, Expr::Var(identifier.to_string())))
}

fn parse_colon_method(input: &str) -> IResult<&str, (String, Vec<Expr>)> {
    let (input, _) = eat_ws(tag(":"))(input)?;
    let (input, method_name) = eat_ws(parse_identifier)(input)?;
    let (input, args) =
        delimited(eat_ws(tag("(")), parse_arg_list_optional, eat_ws(tag(")")))(input)?;
    Ok((input, (method_name.to_string(), args)))
}

fn is_reserved_keyword(s: &str) -> bool {
    matches!(
        s,
        "let" | "if" | "else" | "return" | "fn" | "pb" | "function"
            | "output" | "mod" | "int" | "float" | "bool"
            | "with" | "confidence" | "true" | "false"
            | "Certain" | "Uncertain" | "and" | "or" | "not"
            | "map" | "distribution_of" | "within"
            | "enum" | "bind" | "step"
    )
}

// ── Literals ──────────────────────────────────────────────────────────────────

fn parse_number_literal(input: &str) -> IResult<&str, Expr> {
    let (input, int_part) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let (input, maybe_frac) = opt(pair(
        tag("."),
        take_while1(|c: char| c.is_ascii_digit()),
    ))(input)?;
    match maybe_frac {
        Some((_, frac)) => {
            let s = format!("{}.{}", int_part, frac);
            Ok((input, Expr::Float(f64::from_str(&s).unwrap())))
        }
        None => Ok((input, Expr::Int(i64::from_str(int_part).unwrap()))),
    }
}

/// Parses a raw float value (used for `with confidence >= <float>`).
fn parse_float_value(input: &str) -> IResult<&str, f64> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, int_part) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let (input, maybe_frac) = opt(pair(
        tag("."),
        take_while1(|c: char| c.is_ascii_digit()),
    ))(input)?;
    let s = format!(
        "{}{}{}",
        sign.unwrap_or(""),
        int_part,
        maybe_frac.map(|(_, f)| format!(".{}", f)).unwrap_or_default()
    );
    Ok((input, f64::from_str(&s).unwrap()))
}

// ── Public Entry Point ────────────────────────────────────────────────────────

pub fn parse(input: &str) -> Vec<ProgramItem> {
    match parse_program(input) {
        Ok((remaining, items)) => {
            if !remaining.trim().is_empty() {
                // Compute line number of the first unparsed character.
                let consumed_len = input.len().saturating_sub(remaining.len());
                let consumed = &input[..consumed_len];
                let line = consumed.lines().count().max(1);
                // Show the first non-whitespace problem token.
                let trimmed = remaining.trim_start();
                let snippet: String = trimmed.chars().take(30).collect();
                let snippet = snippet.lines().next().unwrap_or(&snippet);
                panic!("Parse error on line {}: could not parse '{}'", line, snippet);
            }
            items
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let consumed_len = input.len().saturating_sub(e.input.len());
            let consumed = &input[..consumed_len];
            let line = consumed.lines().count().max(1);
            let trimmed = e.input.trim_start();
            let snippet: String = trimmed.chars().take(30).collect();
            let snippet = snippet.lines().next().unwrap_or(&snippet);
            panic!("Parse error on line {}: unexpected '{}'", line, snippet);
        }
        Err(nom::Err::Incomplete(_)) => {
            panic!("Parse error: unexpected end of input");
        }
    }
}
