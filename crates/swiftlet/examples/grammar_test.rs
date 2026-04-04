use std::sync::Arc;
use swiftlet::preclude::*;

fn main() {
    let grammar = r#"
    s: e b e
    e: ("w" | "x") b
    b: f? "y"
    f: "z""#;

    let conf = Arc::new(ParserConfig {
        start: "s".to_string(),
        debug: true,
        algorithm: Algorithm::Earley,
        ..Default::default()
    });

    let parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    // [wx]z?yz?y[wx]z?y
    let text = "xyzyxy";
    match parser.parse(text) {
        Ok(parsed) => parsed.pretty_print(),
        Err(e) => panic!("{}", e),
    }
}
