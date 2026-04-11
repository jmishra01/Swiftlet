//! # Real-world Patterns
//!
//! Parses several common real-world regex patterns and prints a summary of
//! what each top-level AST node represents, showing how a real tool could use
//! this library to analyse or validate patterns before compiling them.
//!
//! Run with:
//!   cargo run -p re-parser --example real_world_patterns

use re_parser::ast::{GroupKind, Regex};
use re_parser::parse;

/// Recursively counts capturing groups (both numbered and named).
fn count_captures(node: &Regex) -> usize {
    match node {
        Regex::Group(inner, kind) => {
            let self_count = matches!(kind, GroupKind::Capturing | GroupKind::Named(_)) as usize;
            self_count + count_captures(inner)
        }
        Regex::Quantifier(inner, _, _) => count_captures(inner),
        Regex::Concat(nodes) | Regex::Alternation(nodes) => {
            nodes.iter().map(count_captures).sum()
        }
        _ => 0,
    }
}

/// Returns true if the pattern is anchored at both ends (^ ... $).
fn is_fully_anchored(node: &Regex) -> bool {
    let Regex::Concat(nodes) = node else {
        return false;
    };
    let first = nodes.first().map(|n| matches!(n, Regex::Anchor(re_parser::ast::Anchor::Start)));
    let last = nodes.last().map(|n| matches!(n, Regex::Anchor(re_parser::ast::Anchor::End)));
    first.unwrap_or(false) && last.unwrap_or(false)
}

/// Collect names of named groups.
fn named_groups(node: &Regex, out: &mut Vec<String>) {
    match node {
        Regex::Group(inner, GroupKind::Named(name)) => {
            out.push(name.clone());
            named_groups(inner, out);
        }
        Regex::Group(inner, _) => named_groups(inner, out),
        Regex::Quantifier(inner, _, _) => named_groups(inner, out),
        Regex::Concat(nodes) | Regex::Alternation(nodes) => {
            for n in nodes {
                named_groups(n, out);
            }
        }
        _ => {}
    }
}

fn report(label: &str, pattern: &str) {
    println!("── {label} ──");
    println!("   pattern  : {pattern}");

    match parse(pattern) {
        Err(e) => println!("   ERROR     : {e}"),
        Ok(ast) => {
            let captures = count_captures(&ast);
            let anchored = is_fully_anchored(&ast);
            let mut names = Vec::new();
            named_groups(&ast, &mut names);

            println!("   captures  : {captures}");
            println!("   anchored  : {anchored}");
            if !names.is_empty() {
                println!("   named     : {}", names.join(", "));
            }
        }
    }
    println!();
}

fn main() {
    report(
        "IPv4 address",
        r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$",
    );

    report(
        "ISO 8601 date",
        r"^(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})$",
    );

    report(
        "Email (simplified)",
        r"^[\w.+-]+@[\w-]+\.[\w.]+$",
    );

    report(
        "Semantic version",
        r"^(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:-(?P<pre>[a-zA-Z0-9.]+))?$",
    );

    report(
        "HTTP method",
        r"^(?:GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)$",
    );

    report(
        "Hex colour",
        r"^#(?:[0-9a-fA-F]{3}){1,2}$",
    );

    report(
        "URL (basic)",
        r"^(?P<scheme>https?)://(?P<host>[^/]+)(?P<path>/[^\s]*)?$",
    );

    // Example: intentionally broken pattern
    report("Invalid escape", r"^\zip$");
}
