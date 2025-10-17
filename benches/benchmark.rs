use barat::grammar::Algorithm;
use barat::{Barat, ParserOption};
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;


const GRAMMAR_AND_TEXT: [(&str, &str); 3] = [
    (
        r#"
        start: expr
        expr: expr "+" number | number
        number: INT
        %import (WS, INT)
        %ignore WS
        "#,
        "1+2+3+4+5+6+7+8+9+1+2+3+4+5+6+7+8+9"
    ),
    (
        r#"
        start: expr
        expr: addition | number
        addition: expr "+" number
        number: number DIGIT | DIGIT
        DIGIT: "0" .. "9"
        %import WS
        %ignore WS
        "#,
        "1+2+3+4+5+6+7+8+9+1+2+3+4+5+6+7+8+9"
    ),
    (
        r#"
        start: expr
        ?expr: expr ("+" | "-" ) factor | factor
        ?factor: factor ("*" | "/") number | number
        number: number? DIGIT
        DIGIT: "0" .. "9"
        %ignore WS
        "#,
        "1*2+3/4+5/6*7+8/9"
    )
];


fn bench_func(c: &mut Criterion) {
    for (i, (g, t)) in GRAMMAR_AND_TEXT.iter().enumerate() {
        // ------- CLR -------- //
        let clr_bench_name = format!("Grammar {} | CLR", i);
        c.bench_function(clr_bench_name.as_str(), |b| b.iter(|| {
            let conf = Arc::new(ParserOption { algorithm: Algorithm::CLR, ..Default::default() });
            let mut clr_parser = Barat::from_string(g.to_string(), conf);
            let _ = clr_parser.parse(t);
        }));

        // ------- EARLEY -------- //
        let earley_bench_name = format!("Grammar {} | EARLEY", i);
        c.bench_function(earley_bench_name.as_str(), |b| b.iter(|| {
            let conf = Arc::new(ParserOption::default());
            let mut parser = Barat::from_string(g.to_string(), conf);
            let _ = parser.parse(t);
        }));
    }
}


criterion_group!(benches, bench_func);
criterion_main!(benches);
