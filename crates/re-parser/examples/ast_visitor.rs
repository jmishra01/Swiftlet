//! # AST Visitor
//!
//! Shows how to walk the parsed AST recursively.  Two visitors are implemented:
//!
//! 1. **`count_nodes`** — counts every node in the tree.
//! 2. **`collect_literals`** — harvests every literal character in order,
//!    which lets you reconstruct the "fixed" parts of a pattern.
//!
//! Run with:
//!   cargo run -p re-parser --example ast_visitor

use re_parser::ast::{CharClassItem, Regex};
use re_parser::parse;

// visitor 1: count nodes

fn count_nodes(node: &Regex) -> usize {
    match node {
        Regex::Literal(_) | Regex::AnyChar | Regex::Anchor(_) | Regex::EscapeClass(_) => 1,
        Regex::CharClass(_) => 1,
        Regex::Group(inner, _) => 1 + count_nodes(inner),
        Regex::Quantifier(inner, _, _) => 1 + count_nodes(inner),
        Regex::Concat(nodes) | Regex::Alternation(nodes) => {
            1 + nodes.iter().map(count_nodes).sum::<usize>()
        }
    }
}

// visitor 2: collect literal characters (left-to-right, depth-first)

fn collect_literals(node: &Regex, out: &mut Vec<char>) {
    match node {
        Regex::Literal(c) => out.push(*c),
        Regex::AnyChar | Regex::Anchor(_) | Regex::EscapeClass(_) => {}
        Regex::CharClass(cls) => {
            // Collect literal items inside the class
            for item in &cls.items {
                if let CharClassItem::Literal(c) = item {
                    out.push(*c);
                }
            }
        }
        Regex::Group(inner, _) => collect_literals(inner, out),
        Regex::Quantifier(inner, _, _) => collect_literals(inner, out),
        Regex::Concat(nodes) | Regex::Alternation(nodes) => {
            for n in nodes {
                collect_literals(n, out);
            }
        }
    }
}

// visitor 3: pretty-print with indentation

fn pretty_print(node: &Regex, indent: usize) {
    let pad = "  ".repeat(indent);
    match node {
        Regex::Literal(c) => println!("{pad}Literal({c:?})"),
        Regex::AnyChar => println!("{pad}AnyChar"),
        Regex::Anchor(a) => println!("{pad}Anchor({a:?})"),
        Regex::EscapeClass(ec) => println!("{pad}EscapeClass({ec:?})"),
        Regex::CharClass(cls) => {
            let neg = if cls.negated { "negated" } else { "" };
            println!("{pad}CharClass[{neg}] ({} items)", cls.items.len());
        }
        Regex::Group(inner, kind) => {
            println!("{pad}Group({kind:?})");
            pretty_print(inner, indent + 1);
        }
        Regex::Quantifier(inner, kind, greedy) => {
            let mode = if *greedy { "greedy" } else { "lazy" };
            println!("{pad}Quantifier({kind:?}, {mode})");
            pretty_print(inner, indent + 1);
        }
        Regex::Concat(nodes) => {
            println!("{pad}Concat ({} children)", nodes.len());
            for n in nodes {
                pretty_print(n, indent + 1);
            }
        }
        Regex::Alternation(nodes) => {
            println!("{pad}Alternation ({} branches)", nodes.len());
            for n in nodes {
                pretty_print(n, indent + 1);
            }
        }
    }
}

fn analyse(pattern: &str) {
    println!("┌─ Pattern: {pattern}");
    let ast = parse(pattern).unwrap();

    let total = count_nodes(&ast);
    let mut lits = Vec::new();
    collect_literals(&ast, &mut lits);
    let lit_str: String = lits.iter().collect();

    println!("│  nodes:    {total}");
    println!("│  literals: {lit_str:?}");
    println!("└─ tree:");
    pretty_print(&ast, 1);
    println!();
}

fn main() {
    analyse(r"\d{4}-\d{2}-\d{2}");          // ISO date skeleton
    analyse(r"(?P<proto>https?)://\S+");      // simple URL prefix
    analyse(r"^[A-Z][a-z]+(?: [A-Z][a-z]+)*$");  // capitalized words
}
