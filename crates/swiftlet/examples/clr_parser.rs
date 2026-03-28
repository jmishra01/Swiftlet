use std::sync::Arc;
use std::time::Instant;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn main() {
    let grammars = [
        r#"
        s: a a
        a: A a | B
        A: "A"
        B: "B"
        %import WS
        %ignore WS
        "#,
        r#"
        s: a a
        a: "A" a | "B"
        %import WS
        %ignore WS
        "#,
    ];

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    for grammar in grammars {
        let t1 = Instant::now();
        let parser = Swiftlet::from_string(grammar, conf.clone())
            .expect("failed to build parser");
        for text in ["ABAB", "ABAAAB"].iter() {
            let t11 = Instant::now();
            if let Ok(parsed) = parser.parse(text) {
                parsed.print();
            }
            let t22 = Instant::now();
            println!("\tparsing test: {:?}", t22 - t11);
        }
        let t2 = Instant::now();
        println!("Loop: {:?}", t2 - t1);
    }
}
