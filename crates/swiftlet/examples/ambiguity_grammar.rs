use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{Ambiguity, ParserConfig, Swiftlet};

/// TODO: Some time below grammar failed

fn main() {
    let text = r#"
        e: t
        x: t "+" t
        y: t "-" t
        t: x | y | i
        i: "id"
        %import WS
        %ignore WS
        "#;

    let conf = Arc::new(ParserConfig {
        start: "e".to_string(),
        algorithm: Algorithm::Earley,
        ambiguity: Ambiguity::Explicit,
        debug: true,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_str(text)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");

    match text_parser.parse("id + id - id") {
        Ok(res) => res.pretty_print(),
        Err(e) => {
            eprintln!("{e}");
        }
    }
}
