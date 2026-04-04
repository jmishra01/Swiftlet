use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};
fn main() {
    let grammar = r#"
    s: "A" _NL "B"
    _NL: /(\r?\n)+/
    "#;

    let texts = [
        r#"A
B"#, "A\nB",
    ];

    let conf = Arc::new(ParserConfig {
        algorithm: Algorithm::Earley,
        debug: true,
        start: "s".to_string(),
        ..Default::default()
    });
    let text_parser = Swiftlet::from_str(grammar)
        .map(|grammar| grammar.parser(conf))
        .expect("failed to build parser");
    for text in texts {
        let ast = text_parser.parse(text);
        ast.unwrap().pretty_print();
    }
}
