
macro_rules! test_case {
        ($fn_name:ident, $grammar:expr, $text:expr, $start:expr, $algorithm:expr) => {
            #[test]
            fn $fn_name() {
            let parser_opt = Arc::new(
                ParserOption {
                    algorithm: $algorithm,
                    start: $start.to_string(),
                    ..Default::default()
                }
            );
            match Swiftlet::from_string($grammar, parser_opt) {
                Ok(parsed) => {
                    assert!(parsed.parse($text).is_ok());
                },
                Err(err) => {
                    debug_assert!(false, "{:?}", err);
                }
            }
        }
        }
    }

macro_rules! multi_test {
        ($fn_name_1:ident, $fn_name_2:ident, $grammar:expr, $text:expr, $start:expr, $algo1:expr, $algo2:expr) => {
            test_case!($fn_name_1, $grammar, $text, $start, $algo1);

            test_case!($fn_name_2, $grammar, $text, $start, $algo2);
    }
}
