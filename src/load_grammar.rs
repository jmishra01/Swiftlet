use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::builder::GrammarBuilder;
use crate::common::get_common_terminals;
use crate::grammar::{create_rules, Algorithm, Rule};
use crate::lexer::{LexerConf, Symbol, TerminalDef};
use crate::parser_frontends::ParserConf;
use crate::parser_frontends::ParserFrontend;
use crate::transform::Transformer;
use crate::{terminal_def, ParserOption};

static RULES: LazyLock<HashMap<Arc<Symbol>, Vec<Arc<Rule>>>> = LazyLock::new(get_rules);

static TERMINALS: LazyLock<Vec<Arc<TerminalDef>>> = LazyLock::new(get_terminals);

pub(crate) static PARSER: LazyLock<Arc<ParserFrontend>> = LazyLock::new(get_parser);

pub(crate) static GRAMMAR_BUILDER: LazyLock<GrammarBuilder> = LazyLock::new(|| {
    let tp_conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    GrammarBuilder::new(PARSER.clone(), tp_conf)
});

const _RE_FLAGS: &str = "imslux";

pub fn get_terminals() -> Vec<Arc<TerminalDef>> {
    let _regex: String = format!(r"/(?!/)(\\/|\\\\|[^/])*?/[{_RE_FLAGS}]*");
    let terminals = vec![
        terminal_def!("_COLON", ":", false, false),
        terminal_def!("_OR", r"|", false, false),
        terminal_def!("_DOT", r"\.(?!\.)", true, false),
        terminal_def!("_DOT_DOT", r"..", false, false),
        terminal_def!("RULE", r"(_|\?)?[a-z][_a-z0-9]*", true, false),
        terminal_def!("TERMINAL", "_?[A-Z][_A-Z0-9]*", true, false),
        terminal_def!("STRING", r#""(\\"|\\|[^"\n])*?"i?"#, true, false),
        terminal_def!("REGEXP", _regex.as_str(), true, false),
        terminal_def!("_NL_OR", r"(\r?\n)+\s*\|", true, false),
        terminal_def!("_NL", r"(\r?\n)+\s*", true, false),
        terminal_def!("WS", r"[ \t]+", true, false),
        terminal_def!("BACKSLASH", r"\\[ ]*\n", true, false),
        terminal_def!("_TO", "->", false, false),
        terminal_def!("_IGNORE", r"%ignore", false, false),
        terminal_def!("_IMPORT", r"%import", false, false),
        terminal_def!("NUMBER", r"[+-]?\d+",true, false),
        terminal_def!("TILDE", "~", true, false),
        terminal_def!("_COMMA", ",", true, false),
        terminal_def!("_COLON", ":", false, false),
        terminal_def!("_OR", r"|", false, false),
        terminal_def!("_LPAR", "(", false, false),
        terminal_def!("_RPAR", ")", false, false),
        terminal_def!("_LBAR", "[", false, false),
        terminal_def!("_RBAR", "]", false, false),
        terminal_def!("_LBRACE", "{", false, false),
        terminal_def!("_RBRACE", "}", false, false),
        terminal_def!("OP", r"[+*]|[?](?![a-z_])", true, false),
    ];
    terminals
}

pub fn get_rules() -> HashMap<Arc<Symbol>, Vec<Arc<Rule>>> {
    create_rules([
        ("start", vec!["_list"]),
        ("_list", vec!["_item", "_item _list"]),
        ("_item", vec!["rule", "term", "ignore", "import"]),
        (
            "rule",
            vec![
                "non_terminal _COLON expansions _NL",
                "non_terminal priority _COLON expansions _NL",
            ],
        ),
        ("term", vec!["terminal _COLON expansions _NL"]),
        ("priority", vec!["_DOT NUMBER"]),
        ("?expansions", vec!["or_expansion"]),
        (
            "or_expansion",
            vec![
                "expansion",
                "or_expansion _OR expansion",
                "or_expansion _NL_OR expansion",
            ],
        ),
        ("?expansion", vec!["alias", "_expansion"]),
        ("alias", vec!["_expansion _TO non_terminal"]),
        ("_expansion", vec!["_expansion _expr", "_expr"]),
        ("_expr", vec!["atom", "op_expansion"]),
        ("op_expansion", vec!["atom OP"]),
        ("?atom", vec!["pars", "maybe", "value"]),
        (
            "?value",
            vec!["terminal", "non_terminal", "string", "range", "regex"],
        ),
        ("?pars", vec!["_LPAR expansions _RPAR"]),
        ("maybe", vec!["_LBAR expansions _RBAR"]),
        ("terminal", vec!["TERMINAL"]),
        ("non_terminal", vec!["RULE"]),
        ("string", vec!["STRING"]),
        ("regex", vec!["REGEXP"]),
        ("range", vec!["STRING _DOT_DOT STRING"]),
        ("ignore", vec!["_IGNORE expansions _NL"]),
        ("?name", vec!["terminal", "non_terminal"]),
        (
            "import",
            vec![
                "_IMPORT _import_args _NL",
                "_IMPORT _LPAR name_list _RPAR _NL",
            ],
        ),
        ("_import_args", vec!["name", "_import_args _DOT name"]),
        ("name_list", vec!["_name_list"]),
        ("_name_list", vec!["name", "name_list _COMMA name"]),
    ])
}

pub fn get_parser() -> Arc<ParserFrontend> {
    let parser_conf = Arc::new(ParserConf::new(
        RULES.clone(),
        Vec::from(["WS".to_string(), "COMMENT".to_string()]),
    ));
    let lexer_conf = Arc::new(LexerConf {
        terminals: TERMINALS.clone(),
    });

    Arc::new(ParserFrontend::new(lexer_conf, parser_conf))
}

pub fn load_grammar(grammar: String) -> Arc<ParserFrontend> {
    let tree = GRAMMAR_BUILDER.parse(grammar.as_str());

    if tree.is_err() {
        panic!("Failed to parse grammar. Error: {}", tree.unwrap_err());
    }

    let tree = tree.unwrap();

    #[cfg(feature = "debug")]
    {
        println!("{}", "-".repeat(50));
        println!("{}AST of Grammar", " ".repeat(18));
        println!("{}", "-".repeat(50));
        tree.pretty_print();
        println!("{}", "-".repeat(50));
    }

    let mut transformer = Transformer::new(get_common_terminals());
    transformer.transform(&tree);
    transformer.sort_terminals();

    let rules = transformer.get_grammar();

    #[cfg(feature = "debug")]
    {
        transformer.print_terminals();
        transformer.print_grammar();
    }

    let parser_conf = Arc::new(ParserConf::new(rules, transformer.get_ignores()));
    let lexer_conf = Arc::new(LexerConf::new(transformer.get_terminal()));

    Arc::new(ParserFrontend::new(lexer_conf, parser_conf))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! make_test_case {
        ($fn_name:ident, $grammar:expr, $left: expr) => {
            #[test]
            fn $fn_name() {
                let text = $grammar;
                let left = $left;
                if let Ok(ast) = GRAMMAR_BUILDER.parse(text) {
                    let right = ast.get_text();
                    assert_eq!(left, right);
                } else {
                    panic!("Failed to generate AST.");
                }
            }
        };
    }

    make_test_case!(
        test_single_word,
        r#"
        start: T
        T: "expr"
        "#,
        r#"start([rule([non_terminal([start]), or_expansion([terminal([T])])]), term([terminal([T]), or_expansion([string(["expr"])])])])"#
    );

    make_test_case!(
        test_string_double_quotes,
        r#"
        start: T
        T: "ex\"pr"
        "#,
        r#"start([rule([non_terminal([start]), or_expansion([terminal([T])])]), term([terminal([T]), or_expansion([string(["ex\"pr"])])])])"#
    );

    make_test_case!(
        test_ws_ignore,
        r#"
        start: T
        T: "a"
        %import WS
        %ignore WS
        "#,
        r#"start([rule([non_terminal([start]), or_expansion([terminal([T])])]), term([terminal([T]), or_expansion([string(["a"])])]), import([terminal([WS])]), ignore([or_expansion([terminal([WS])])])])"#
    );

    make_test_case!(
        test_abbab,
        r#"
        s: e
        e: e t
            | t
        t: t "a"
            | "b"
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([or_expansion([expansion([non_terminal([e]), non_terminal([t])])]), non_terminal([t])])]), rule([non_terminal([t]), or_expansion([or_expansion([expansion([non_terminal([t]), string(["a"])])]), string(["b"])])])])"#
    );

    make_test_case!(
        test_priority,
        r#"
        s: e
        ?e: t
        t: "hello"
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([?e]), or_expansion([non_terminal([t])])]), rule([non_terminal([t]), or_expansion([string(["hello"])])])])"#
    );

    make_test_case!(
        test_addition_range,
        r#"
        s: e
        e: e "+" t
        t: "0".."9"
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([expansion([non_terminal([e]), string(["+"]), non_terminal([t])])])]), rule([non_terminal([t]), or_expansion([range(["0", "9"])])])])"#
    );

    make_test_case!(
        test_to_add,
        r#"
        s: e
        e: e "+" t -> add
        t: "0".."9"
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([alias([non_terminal([e]), string(["+"]), non_terminal([t]), non_terminal([add])])])]), rule([non_terminal([t]), or_expansion([range(["0", "9"])])])])"#
    );

    make_test_case!(
        test_to_add_sub,
        r#"
        s: e
        e: e "+" t -> add
            | e "-" t -> sub
            | t
        t: "0".."9"
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([or_expansion([or_expansion([alias([non_terminal([e]), string(["+"]), non_terminal([t]), non_terminal([add])])]), alias([non_terminal([e]), string(["-"]), non_terminal([t]), non_terminal([sub])])]), non_terminal([t])])]), rule([non_terminal([t]), or_expansion([range(["0", "9"])])])])"#
    );


    make_test_case!(
        test_optional_with_or,
        r#"
        s: e
        e: (e ("+" | "-"))? t
        t: "0".."9"
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([expansion([op_expansion([or_expansion([expansion([non_terminal([e]), or_expansion([or_expansion([string(["+"])]), string(["-"])])])]), ?]), non_terminal([t])])])]), rule([non_terminal([t]), or_expansion([range(["0", "9"])])])])"#
    );


    make_test_case!(
        test_import_and_comment,
        r#"
        s: e
        e: (e ("+" | "-"))? t
        t: INT
        %import (WS, INT)
        %ignore WS
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([expansion([op_expansion([or_expansion([expansion([non_terminal([e]), or_expansion([or_expansion([string(["+"])]), string(["-"])])])]), ?]), non_terminal([t])])])]), rule([non_terminal([t]), or_expansion([terminal([INT])])]), import([name_list([name_list([terminal([WS])]), terminal([INT])])]), ignore([or_expansion([terminal([WS])])])])"#
    );


    make_test_case!(
        test_calculate,
        r#"
        s: e
        e: (e ("+" | "-"))? t
        t: (t ("*" | "\"))? d
        d: "(" e ")" | v
        v: INT
        %import (WS, INT)
        %ignore WS
        "#,
        r#"start([rule([non_terminal([s]), or_expansion([non_terminal([e])])]), rule([non_terminal([e]), or_expansion([expansion([op_expansion([or_expansion([expansion([non_terminal([e]), or_expansion([or_expansion([string(["+"])]), string(["-"])])])]), ?]), non_terminal([t])])])]), rule([non_terminal([t]), or_expansion([expansion([op_expansion([or_expansion([expansion([non_terminal([t]), or_expansion([or_expansion([string(["*"])]), string(["\"])])])]), ?]), non_terminal([d])])])]), rule([non_terminal([d]), or_expansion([or_expansion([expansion([string(["("]), non_terminal([e]), string([")"])])]), non_terminal([v])])]), rule([non_terminal([v]), or_expansion([terminal([INT])])]), import([name_list([name_list([terminal([WS])]), terminal([INT])])]), ignore([or_expansion([terminal([WS])])])])"#
    );
}
