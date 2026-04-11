//! # Basic Parsing
//!
//! This example shows the fundamentals: parsing a pattern and inspecting the
//! resulting AST node variants.
//!
//! Run with:
//!   cargo run -p re-parser --example basic_parsing

use re_parser::ast::Regex;
use re_parser::parse;

fn main() {
    // ── 1. A single literal ──────────────────────────────────────────────────
    let ast = parse("a").unwrap();
    println!("Pattern: \"a\"");
    println!("AST:     {ast:?}\n");

    // ── 2. A concatenation of literals ───────────────────────────────────────
    let ast = parse("hello").unwrap();
    println!("Pattern: \"hello\"");
    match &ast {
        Regex::Concat(nodes) => println!("AST:     Concat of {} nodes", nodes.len()),
        other => println!("AST:     {other:?}"),
    }
    println!();

    // ── 3. Alternation (a|b|c) ───────────────────────────────────────────────
    let ast = parse("cat|dog|bird").unwrap();
    println!("Pattern: \"cat|dog|bird\"");
    match &ast {
        Regex::Alternation(branches) => {
            println!("AST:     Alternation with {} branches:", branches.len());
            for branch in branches {
                println!("           {branch:?}");
            }
        }
        other => println!("AST:     {other:?}"),
    }
    println!();

    // ── 4. Any character and anchors ─────────────────────────────────────────
    for pattern in [".", "^", "$", r"\b"] {
        let ast = parse(pattern).unwrap();
        println!("Pattern: {pattern:?}  =>  {ast:?}");
    }
    println!();

    // ── 5. Error handling ────────────────────────────────────────────────────
    let patterns = ["(unclosed", r"\z"];
    for p in patterns {
        match parse(p) {
            Ok(ast) => println!("Pattern: {p:?}  =>  {ast:?}"),
            Err(e) => println!("Pattern: {p:?}  =>  ERROR: {e}"),
        }
    }
}
