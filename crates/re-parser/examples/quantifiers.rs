//! # Quantifiers
//!
//! Demonstrates every quantifier form — greedy and lazy — and how to read them
//! from the AST.
//!
//! Run with:
//!   cargo run -p re-parser --example quantifiers

use re_parser::ast::{QuantKind, Regex};
use re_parser::parse;

fn describe_quant(kind: &QuantKind, greedy: bool) -> String {
    let mode = if greedy { "greedy" } else { "lazy" };
    let desc = match kind {
        QuantKind::ZeroOrMore => "zero-or-more (*)".to_owned(),
        QuantKind::OneOrMore => "one-or-more (+)".to_owned(),
        QuantKind::ZeroOrOne => "zero-or-one (?)".to_owned(),
        QuantKind::Exactly(n) => format!("exactly {n} ({{{n}}})"),
        QuantKind::AtLeast(n) => format!("at-least {n} ({{{n},}})"),
        QuantKind::Between(n, m) => format!("between {n} and {m} ({{{n},{m}}})"),
    };
    format!("{desc}, {mode}")
}

fn print_quantifier(pattern: &str) {
    let ast = parse(pattern).unwrap();
    match &ast {
        Regex::Quantifier(inner, kind, greedy) => {
            println!(
                "  {pattern:10}  =>  applied to {:?},  {}",
                inner,
                describe_quant(kind, *greedy)
            );
        }
        other => println!("  {pattern:10}  =>  {other:?}"),
    }
}

fn main() {
    println!("=== Greedy quantifiers ===");
    print_quantifier("a*");
    print_quantifier("a+");
    print_quantifier("a?");
    print_quantifier("a{3}");
    print_quantifier("a{2,}");
    print_quantifier("a{1,5}");

    println!("\n=== Lazy (non-greedy) quantifiers ===");
    print_quantifier("a*?");
    print_quantifier("a+?");
    print_quantifier("a??");
    print_quantifier("a{2,6}?");

    println!("\n=== Quantifier applied to a group ===");
    let pattern = r"(\d+)?";
    let ast = parse(pattern).unwrap();
    println!("  {pattern}  =>  {ast:#?}");
}
