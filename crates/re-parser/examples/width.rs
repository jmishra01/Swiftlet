//! # Match-width Analysis
//!
//! Demonstrates `.min_width()` and `.max_width()` — methods on the parsed
//! `Regex` AST node that compute how many characters a pattern can consume
//! without running it against any input.
//!
//! Run with:
//!   cargo run -p re-parser --example width

use re_parser::parse;
use re_parser::pattern_width; // convenience: parse + width in one call

fn report(pattern: &str) {
    match parse(pattern) {
        Err(e) => println!("  {pattern:40}  ERROR: {e}"),
        Ok(ast) => {
            let min = ast.min_width();
            let max = ast.max_width();
            let max_str = max.map_or("∞".to_owned(), |n| n.to_string());
            println!("  {pattern:40}  min={min}  max={max_str}");
        }
    }
}

fn main() {
    println!("=== Fixed-width patterns ===");
    report("abc");
    report(r"\d{4}-\d{2}-\d{2}");           // ISO date → 10
    report(r"#[0-9a-fA-F]{6}");             // CSS hex colour → 7
    report(r"[A-Z]{2}[0-9]{4}");            // plate → 6

    println!("\n=== Variable but bounded patterns ===");
    report(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"); // IPv4 → 7..=15
    report(r"https?://");                    // 7..=8
    report(r"[a-zA-Z]{2,8}");               // 2..=8
    report(r"colou?r");                     // 5..=6

    println!("\n=== Unbounded patterns ===");
    report(r"\w+");                          // 1..∞
    report(r".*");                           // 0..∞
    report(r"\S+@\S+\.\S+");               // 5..∞
    report(r"(?:ab)+");                     // 2..∞

    println!("\n=== Anchors and lookarounds are zero-width ===");
    report(r"^hello$");                     // anchors → still 5
    report(r"foo(?=bar)");                  // lookahead → 3
    report(r"(?<=\d)px");                   // lookbehind → 2
    report(r"\bword\b");                    // \b → 4

    println!("\n=== Alternation ===");
    report(r"cat|dog");                     // exactly 3
    report(r"a|bb|ccc");                    // 1..=3
    report(r"yes|no");                      // 2..=3
    report(r"x|\d+");                       // 1..∞

    // ── using pattern_width convenience function ───────────────────────────
    println!("\n=== pattern_width() convenience wrapper ===");
    let w = pattern_width(r"a{2,5}").unwrap();
    println!("  a{{2,5}} => {w}");
    println!("    is_fixed:     {}", w.is_fixed());
    println!("    is_nullable:  {}", w.is_nullable());
    println!("    is_unbounded: {}", w.is_unbounded());

    // ── calling methods on a sub-node ──────────────────────────────────────
    println!("\n=== Inspecting sub-nodes ===");
    let ast = parse(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    println!("  full pattern  min={} max={:?}", ast.min_width(), ast.max_width());

    // Walk the top-level Concat children manually
    if let re_parser::ast::Regex::Concat(nodes) = &ast {
        for (i, node) in nodes.iter().enumerate() {
            println!(
                "  child[{i}]  min={}  max={:?}  node={node:?}",
                node.min_width(),
                node.max_width(),
            );
        }
    }
}
