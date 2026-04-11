//! # Character Classes
//!
//! Demonstrates parsing `[...]` character class expressions including ranges,
//! negation, escape shorthands inside brackets, and mixed items.
//!
//! Run with:
//!   cargo run -p re-parser --example char_classes

use re_parser::ast::{CharClassItem, EscapeClass, Regex};
use re_parser::parse;

fn describe_item(item: &CharClassItem) -> String {
    match item {
        CharClassItem::Literal(c) => format!("literal({c:?})"),
        CharClassItem::Range(lo, hi) => format!("range({lo:?}..={hi:?})"),
        CharClassItem::EscapeClass(ec) => {
            let name = match ec {
                EscapeClass::Digit => r"\d",
                EscapeClass::NonDigit => r"\D",
                EscapeClass::Word => r"\w",
                EscapeClass::NonWord => r"\W",
                EscapeClass::Space => r"\s",
                EscapeClass::NonSpace => r"\S",
            };
            format!("escape({name})")
        }
    }
}

fn print_class(pattern: &str) {
    let ast = parse(pattern).unwrap();
    match &ast {
        Regex::CharClass(cls) => {
            let negated = if cls.negated { "negated, " } else { "" };
            let items: Vec<_> = cls.items.iter().map(describe_item).collect();
            println!("  {pattern:15}  =>  [{negated}{}]", items.join(", "));
        }
        other => println!("  {pattern:15}  =>  {other:?}"),
    }
}

fn main() {
    println!("=== Literal sets ===");
    print_class("[abc]");
    print_class("[aeiou]");

    println!("\n=== Negated classes ===");
    print_class("[^abc]");
    print_class("[^0-9]");

    println!("\n=== Character ranges ===");
    print_class("[a-z]");
    print_class("[A-Z]");
    print_class("[0-9]");
    print_class("[a-zA-Z]");

    println!("\n=== Escape shorthands inside brackets ===");
    print_class(r"[\d]");
    print_class(r"[\w\s]");
    print_class(r"[^\W_]");   // word chars minus underscore

    println!("\n=== Mixed ===");
    print_class(r"[a-z\d_]");   // lowercase letter, digit, or underscore
    print_class(r"[^a-z\s]");   // not a lowercase letter and not whitespace

    println!("\n=== Class inside a larger pattern ===");
    let pattern = r"[A-Z][a-z]+";
    println!("  Pattern: {pattern}");
    println!("  AST:\n{:#?}", parse(pattern).unwrap());
}
