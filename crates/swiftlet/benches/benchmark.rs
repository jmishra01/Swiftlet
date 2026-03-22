use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

const GRAMMAR_AND_TEXT: [(&str, &str); 3] = [
    (
        r#"
        start: expr
        expr: expr "+" number | number
        number: INT
        %import (WS, INT)
        %ignore WS
        "#,
        "10 + 76 + 10 + 85 + 14 + 6 + 79 + 46 + 58 + 53 + 87 + 77 + 61 + 11 + 80 + 85 + 5 + 1 + 16 + 16 + 64 + 26 + 8 + 75 + 37 + 26 + 78 + 14 + 96 + 73 + 34 + 89 + 33 + 34 + 67 + 49 + 58 + 31 + 17 + 40 + 49 + 77 + 93 + 70 + 33 + 10 + 76 + 42 + 82 + 87 + 29 + 72 + 7 + 2 + 91 + 36 + 91 + 80 + 2 + 29 + 85 + 19 + 76 + 0 + 24 + 31 + 68 + 63 + 96 + 36 + 68 + 58 + 2 + 71 + 29 + 30 + 45 + 49 + 3 + 50 + 83 + 4 + 44 + 19 + 79 + 44 + 71 + 88 + 73 + 34 + 96 + 95 + 74 + 97 + 79 + 57 + 61 + 34 + 67 + 1 + 88",
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
        "10 + 76 + 10 + 85 + 14 + 6 + 79 + 46 + 58 + 53 + 87 + 77 + 61 + 11 + 80 + 85 + 5 + 1 + 16 + 16 + 64 + 26 + 8 + 75 + 37 + 26 + 78 + 14 + 96 + 73 + 34 + 89 + 33 + 34 + 67 + 49 + 58 + 31 + 17 + 40 + 49 + 77 + 93 + 70 + 33 + 10 + 76 + 42 + 82 + 87 + 29 + 72 + 7 + 2 + 91 + 36 + 91 + 80 + 2 + 29 + 85 + 19 + 76 + 0 + 24 + 31 + 68 + 63 + 96 + 36 + 68 + 58 + 2 + 71 + 29 + 30 + 45 + 49 + 3 + 50 + 83 + 4 + 44 + 19 + 79 + 44 + 71 + 88 + 73 + 34 + 96 + 95 + 74 + 97 + 79 + 57 + 61 + 34 + 67 + 1 + 88",
    ),
    (
        r#"
        start: expr
        ?expr: expr ("+" | "-" ) factor | factor
        ?factor: factor ("*" | "/") number | number
        number: number? DIGIT
        DIGIT: "0" .. "9"
        %import WS
        %ignore WS
        "#,
        "93 * 92 * 97 / 34 * 82 + 70 + 83 + 99 / 18 / 53 - 66 - 29 / 21 * 73 - 57 / 55 + 53 / 44 + 65 - 95 * 13 - 89 - 34 - 91 * 52 + 85 - 2 + 88 - 45 - 48 / 50 - 5 * 0 - 1 - 99 / 11 - 57 * 12 * 97 * 48 + 23 + 21 / 89 / 65 * 31 / 2 - 21 * 56 / 90 - 89 - 27 / 84 - 47 + 17 * 98 * 63 - 81 / 60 - 73 + 54 + 9 - 71 * 88 + 11 * 71 * 60 + 25 * 85 + 49 + 74 * 78 + 34 * 17 - 64 + 91 - 78 * 79 * 48 * 20 + 73 - 18 * 41 / 24 * 31 - 80 + 34 + 63 + 80 * 99 - 83 - 91 * 98 * 8 - 42 * 85 - 83 - 21 + 98 / 98 / 19 / 70",
    ),
];

fn bench_func(c: &mut Criterion) {
    for (i, (g, t)) in GRAMMAR_AND_TEXT.iter().enumerate() {
        // ------- CLR -------- //
        let clr_bench_name = format!("Grammar {} | CLR", i);
        c.bench_function(clr_bench_name.as_str(), |b| {
            b.iter(|| {
                let conf = Arc::new(ParserOption {
                    algorithm: Algorithm::CLR,
                    ..Default::default()
                });
                let clr_parser = Swiftlet::from_string(g, conf);
                let _ = clr_parser.parse(t);
            })
        });

        // ------- EARLEY -------- //
        let earley_bench_name = format!("Grammar {} | EARLEY", i);
        c.bench_function(earley_bench_name.as_str(), |b| {
            b.iter(|| {
                let conf = Arc::new(ParserOption::default());
                let parser = Swiftlet::from_string(g, conf);
                let _ = parser.parse(t);
            })
        });
    }
}

criterion_group!(benches, bench_func);
criterion_main!(benches);
