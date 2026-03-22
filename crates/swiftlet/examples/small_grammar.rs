use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let text = r#"
        start: expr
        expr: expr (M | N) | N
        M: "A"
        N: "B"
        %import WS
        %import INT
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        debug: true,
        ..Default::default()
    });

    let text_parser = Swiftlet::from_string(text, conf);
    match text_parser.parse("BABBAABA") {
        Ok(result) => result.pretty_print(),
        Err(err) => {
            panic!("{}", err)
        }
    }
}
