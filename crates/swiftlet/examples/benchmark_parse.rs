use std::env;
use std::sync::Arc;
use std::time::Instant;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

fn grammar() -> &'static str {
    r#"
start: expr
expr: expr "+" term -> add
    | expr "-" term -> sub
    | term
term: term "*" atom -> mul
    | term "/" atom -> div
    | atom
atom: INT
    | "(" expr ")"
%import (WS, INT)
%ignore WS
"#
}

fn build_expression(repetitions: usize) -> String {
    let mut parts = vec!["1".to_string()];
    for index in 2..=(repetitions + 1) {
        let operator = if index % 2 == 0 { "+" } else { "*" };
        parts.push(operator.to_string());
        parts.push(index.to_string());
    }
    parts.join(" ")
}

fn make_parser_option(algorithm: &str) -> Arc<ParserOption> {
    let algorithm = match algorithm {
        "earley" => Algorithm::Earley,
        "clr" => Algorithm::CLR,
        other => panic!("unsupported algorithm: {other}"),
    };

    Arc::new(ParserOption {
        algorithm,
        ..ParserOption::default()
    })
}

fn benchmark_constructor(rounds: usize, algorithm: &str) -> Vec<f64> {
    let mut samples = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let start = Instant::now();
        let _parser = Swiftlet::from_string(grammar(), make_parser_option(algorithm))
            .expect("failed to build parser");
        samples.push(start.elapsed().as_secs_f64());
    }
    samples
}

fn benchmark_parse(rounds: usize, algorithm: &str, text: &str) -> Vec<f64> {
    let parser = Swiftlet::from_string(grammar(), make_parser_option(algorithm))
        .expect("failed to build parser");
    let _ = parser.parse(text).unwrap();

    let mut samples = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let start = Instant::now();
        let _ = parser.parse(text).unwrap();
        samples.push(start.elapsed().as_secs_f64());
    }
    samples
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn format_stats(name: &str, samples: &[f64]) -> String {
    let converted = samples
        .iter()
        .map(|sample| sample * 1000.0)
        .collect::<Vec<_>>();
    format!(
        "{name}: mean={:.3}ms, median={:.3}ms, min={:.3}ms, max={:.3}ms",
        mean(&converted),
        median(&converted),
        converted.iter().copied().fold(f64::INFINITY, f64::min),
        converted.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    )
}

fn parse_flag(name: &str, default: &str) -> String {
    let args = env::args().collect::<Vec<_>>();
    args.windows(2)
        .find_map(|window| {
            if window[0] == name {
                Some(window[1].clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| default.to_string())
}

fn has_flag(name: &str) -> bool {
    env::args().any(|arg| arg == name)
}

fn main() {
    let rounds = parse_flag("--rounds", "200").parse::<usize>().unwrap();
    let repetitions = parse_flag("--repetitions", "50").parse::<usize>().unwrap();
    let algorithm = parse_flag("--algorithm", "earley");
    let json = has_flag("--json");

    let text = build_expression(repetitions);
    let constructor_samples = benchmark_constructor(rounds, algorithm.as_str());
    let parse_samples = benchmark_parse(rounds, algorithm.as_str(), text.as_str());
    let parse_ops_per_sec = rounds as f64 / parse_samples.iter().sum::<f64>();

    if json {
        println!(
            "{{\"library\":\"rust\",\"algorithm\":\"{}\",\"rounds\":{},\"input_tokens\":{},\"construct_mean_ms\":{:.6},\"parse_mean_ms\":{:.6},\"parse_ops_per_sec\":{:.6}}}",
            algorithm,
            rounds,
            text.split_whitespace().count(),
            mean(&constructor_samples) * 1000.0,
            mean(&parse_samples) * 1000.0,
            parse_ops_per_sec
        );
        return;
    }

    println!("algorithm={algorithm}");
    println!("rounds={rounds}");
    println!("input_tokens={}", text.split_whitespace().count());
    println!(
        "{}",
        format_stats("construct", constructor_samples.as_slice())
    );
    println!("{}", format_stats("parse", parse_samples.as_slice()));
    println!("parse_ops_per_sec={parse_ops_per_sec:.2}");
}
