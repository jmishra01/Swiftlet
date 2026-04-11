//! # Groups
//!
//! Shows every group variant the parser supports: capturing, named, non-capturing,
//! and all four lookaround assertions.
//!
//! Run with:
//!   cargo run -p re-parser --example groups

use re_parser::ast::{GroupKind, Regex};
use re_parser::parse;

fn group_kind_label(kind: &GroupKind) -> &'static str {
    match kind {
        GroupKind::Capturing => "capturing",
        GroupKind::Named(_) => "named capturing",
        GroupKind::NonCapturing => "non-capturing",
        GroupKind::LookaheadPos => "positive lookahead",
        GroupKind::LookaheadNeg => "negative lookahead",
        GroupKind::LookbehindPos => "positive lookbehind",
        GroupKind::LookbehindNeg => "negative lookbehind",
    }
}

fn print_group(pattern: &str) {
    let ast = parse(pattern).unwrap();

    // The outermost node might be a Concat if there is content outside the group.
    // Walk one level to find the first Group node for display.
    let group_node = match &ast {
        Regex::Group(_, _) => Some(&ast),
        Regex::Concat(nodes) => nodes.iter().find(|n| matches!(n, Regex::Group(_, _))),
        _ => None,
    };

    match group_node {
        Some(Regex::Group(inner, kind)) => {
            let label = group_kind_label(kind);
            let name_part = if let GroupKind::Named(n) = kind {
                format!(" (name = \"{n}\")")
            } else {
                String::new()
            };
            println!(
                "  {pattern:20}  =>  {label}{name_part}, inner = {:?}",
                inner
            );
        }
        _ => println!("  {pattern:20}  =>  {ast:?}"),
    }
}

fn main() {
    println!("=== Capturing & non-capturing ===");
    print_group("(abc)");
    print_group("(?:abc)");
    print_group("(?P<word>\\w+)");

    println!("\n=== Lookahead assertions ===");
    print_group("foo(?=bar)");   // 'foo' only when followed by 'bar'
    print_group("foo(?!bar)");   // 'foo' only when NOT followed by 'bar'

    println!("\n=== Lookbehind assertions ===");
    print_group("(?<=\\d)px");   // 'px' only when preceded by a digit
    print_group("(?<!\\d)px");   // 'px' only when NOT preceded by a digit

    println!("\n=== Nested groups (full pretty AST) ===");
    let pattern = "(?P<date>(?P<year>\\d{4})-(?P<month>\\d{2}))";
    println!("  Pattern: {pattern}");
    println!("  AST:\n{:#?}", parse(pattern).unwrap());
}
