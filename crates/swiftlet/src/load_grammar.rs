use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::builder::GrammarBuilder;
use crate::common::get_common_terminals;
use crate::error::ParserError;
use crate::grammar::{Algorithm, Rule, create_rules};
use crate::lexer::{LexerConf, RegexFlag, Symbol, TerminalDef};
use crate::parser_frontends::ParserConf;
use crate::parser_frontends::ParserFrontend;
use crate::transform::{RuleCompiler, TerminalCompiler, fetch_terminals};
use crate::{ParserOption, terminal_def};

static RULES: LazyLock<HashMap<Arc<Symbol>, Vec<Arc<Rule>>>> = LazyLock::new(get_rules);

static TERMINALS: LazyLock<Vec<Arc<TerminalDef>>> = LazyLock::new(get_terminals);

pub(crate) static PARSER: LazyLock<Arc<ParserFrontend>> = LazyLock::new(get_parser);

pub static GRAMMAR_BUILDER: LazyLock<GrammarBuilder> = LazyLock::new(|| {
    let tp_conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    GrammarBuilder::new(PARSER.clone(), tp_conf)
});

const _RE_FLAGS: &str = "imsux";

/// Returns terminal definitions used by the grammar-language parser.
pub fn get_terminals() -> Vec<Arc<TerminalDef>> {
    let _regex: String = format!(r"/(?!/)(\\/|\\\\|[^/])*?/[{_RE_FLAGS}]*");
    let terminals = vec![
        terminal_def!("_COLON", ":", 27),
        terminal_def!("_OR", r"|", 26),
        terminal_def!("_DOT", r"\.(?!\.)", RegexFlag::default(), 25),
        terminal_def!("_DOT_DOT", r"..", 24),
        terminal_def!("RULE", r"(_|\?)?[a-z][_a-z0-9]*", RegexFlag::default(), 23),
        terminal_def!("TERMINAL", "_?[A-Z][_A-Z0-9]*", RegexFlag::default(), 22),
        terminal_def!(
            "STRING",
            r#""(\\"|\\|[^"\n])*?"i?"#,
            RegexFlag::default(),
            21
        ),
        terminal_def!("REGEXP", _regex.as_str(), RegexFlag::default(), 20),
        terminal_def!("_NL_OR", r"(\r?\n)+\s*\|", RegexFlag::default(), 19),
        terminal_def!("_NL", r"(\r?\n)+\s*", RegexFlag::default(), 18),
        terminal_def!("WS", r"[ \t]+", RegexFlag::default(), 17),
        terminal_def!("BACKSLASH", r"\\[ ]*\n", RegexFlag::default(), 16),
        terminal_def!("_TO", "->", 15),
        terminal_def!("_IGNORE", r"%ignore", 14),
        terminal_def!("_IMPORT", r"%import", 13),
        terminal_def!("NUMBER", r"[+-]?\d+", RegexFlag::default(), 12),
        terminal_def!("TILDE", "~", RegexFlag::default(), 11),
        terminal_def!("_COMMA", ",", RegexFlag::default(), 10),
        terminal_def!("_COLON", ":", 9),
        terminal_def!("_OR", r"|", 8),
        terminal_def!("_LPAR", "(", 7),
        terminal_def!("_RPAR", ")", 6),
        terminal_def!("_LBAR", "[", 5),
        terminal_def!("_RBAR", "]", 4),
        terminal_def!("_LBRACE", "{", 3),
        terminal_def!("_RBRACE", "}", 2),
        terminal_def!("OP", r"[+*]|[?](?![a-z_])", RegexFlag::default(), 1),
    ];
    terminals
}

/// Returns grammar-language production rules.
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
        (
            "term",
            vec![
                "terminal _COLON expansions _NL",
                "terminal priority _COLON expansions _NL",
            ],
        ),
        ("priority", vec!["_DOT NUMBER"]),
        ("?expansions", vec!["or_expansion"]),
        ("or_expansion", vec!["_or_expansion"]),
        (
            "_or_expansion",
            vec![
                "expansion",
                "_or_expansion _OR expansion",
                "_or_expansion _NL_OR expansion",
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
        ("maybe", vec!["_LBAR _or_expansion _RBAR"]),
        ("terminal", vec!["TERMINAL"]),
        ("non_terminal", vec!["RULE"]),
        ("string", vec!["STRING"]),
        ("regex", vec!["REGEXP"]),
        ("range", vec!["STRING _DOT_DOT STRING"]),
        ("_term_str", vec!["terminal", "string"]),
        ("_terminal", vec!["_term_str", "_terminal _COMMA _term_str"]),
        (
            "ignore",
            vec!["_IGNORE value _NL", "_IGNORE _LPAR _terminal _RPAR _NL"],
        ),
        (
            "import",
            vec!["_IMPORT terminal _NL", "_IMPORT _LPAR _terminal _RPAR _NL"],
        ),
    ])
}

/// Creates the parser frontend used for parsing grammar definitions.
pub fn get_parser() -> Arc<ParserFrontend> {
    // TODO: Add _NL in ignore list
    let parser_conf = Arc::new(ParserConf::new(
        RULES.clone(),
        Vec::from(["WS".to_string(), "COMMENT".to_string()]),
    ));
    let lexer_conf = Arc::new(LexerConf {
        terminals: TERMINALS.clone(),
    });

    Arc::new(ParserFrontend::new(lexer_conf, parser_conf))
}

/// Parses grammar text and transforms it into a runnable parser frontend.
macro_rules! impl_load_grammar {
    ($( $extra_arg:ident : $extra_type:ty ),*) => {
        /// Loads a grammar definition into a ready-to-use parser frontend.
        pub fn load_grammar(grammar: &str, $( $extra_arg : $extra_type ),*) -> Result<Arc<ParserFrontend>, ParserError> {
            let tree = GRAMMAR_BUILDER
                .parse(grammar)
                .map_err(|e| ParserError::GrammarParseError(e.to_string()))?;
            $(
                if $extra_arg.debug {
                    println!("\nAST of Grammar");
                    println!("==============");
                    tree.pretty_print();
                    println!();
                }
            )*

            let common_terminals = get_common_terminals();
            let mut terminals = if let Some(terminals) = tree.get_child_tree("term") {
                let mut terminal_compiler = TerminalCompiler::new(terminals);
                terminal_compiler.compile();
                terminal_compiler.get_terminals()
            } else {
                vec![]
            };

            let mut update_terminals = |arg0: &String| {
                if common_terminals.contains_key(arg0) {
                    terminals.push(common_terminals.get(arg0).unwrap().clone());
                } else {
                    terminals.push(Arc::new(TerminalDef::with_regex(
                        arg0,
                        arg0,
                        RegexFlag::default(),
                        5
                    )
                    ));
                }
            };

            let ignores: Vec<String> = match tree.get_child_tree("ignore") {
                Some(ignores) => {
                    ignores
                    .iter()
                    .map(|ignore| fetch_terminals(ignore))
                    .flatten()
                    .collect::<Vec<_>>()
                },
                None => vec![],
            };
            ignores.iter().for_each(&mut update_terminals);

            if let Some(imports) = tree.get_child_tree("import") {
                imports
                .iter()
                .map(|ignore| fetch_terminals(ignore))
                .flatten()
                .for_each(|arg0: String| update_terminals(&arg0));
            }
            let Some(rules) = tree.get_child_tree("rule") else {
                return Err(ParserError::GrammarParseError(
                    "grammar does not contain any rule definitions".to_string(),
                ));
            };

            let mut transformer = RuleCompiler::new();

            transformer.compile(rules);

            let rules = match transformer.get_grammar() {
                Ok(rules) => {
                    rules
                },
                Err(e) => return Err(e)
            };

            terminals.extend(transformer.get_terminal());
            terminals.sort_by(|first, second| {
                second.priority.cmp(&first.priority)
                .then(
                    second.max_width.cmp(&first.max_width)
                    .then(second.value.len().cmp(&first.value.len()))
                )
            });
            terminals.dedup_by(|first, second| {
                first.get_name() == second.get_name()
                    && first.value == second.value
                    && first.priority == second.priority
                    && first.max_width == second.max_width
            });

            $(
                if $extra_arg.debug {
                    println!("\nTerminals");
                    println!("=========");
                    for t in terminals.iter() {
                        println!("{t:?}");
                    }

                    println!("\nGrammar");
                    println!("=======");
                    for (_, prod) in rules.iter() {
                        for p in prod.iter() {
                            println!("{p:?}");
                        }
                    }
                }
            )*

            let parser_conf = Arc::new(ParserConf::new(rules, ignores));
            let lexer_conf = Arc::new(LexerConf::new(terminals));

            Ok(Arc::new(ParserFrontend::new(lexer_conf, parser_conf)))
        }
    };
}

#[cfg(feature = "debug")]
impl_load_grammar!(parser_option: Arc<ParserOption>);

#[cfg(not(feature = "debug"))]
impl_load_grammar!();

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_grammar(grammar: &str) -> String {
        let mut normalized = grammar
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        normalized.push('\n');
        normalized
    }

    macro_rules! make_test_case {
        ($fn_name:ident, $grammar:expr, $left: expr) => {
            #[test]
            fn $fn_name() {
                let text = normalize_grammar($grammar);
                let left = $left;
                if let Ok(ast) = GRAMMAR_BUILDER.parse(&text) {
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "start")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "T")])])]), Tree("term", [Tree("terminal", [Token(TERMINAL, "T")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""expr"")])])])])"#
    );

    make_test_case!(
        test_string_double_quotes,
        r#"
        start: T
        T: "ex\"pr"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "start")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "T")])])]), Tree("term", [Tree("terminal", [Token(TERMINAL, "T")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""ex\"pr"")])])])])"#
    );

    make_test_case!(
        test_ws_ignore,
        r#"
        start: T
        T: "a"
        %import WS
        %ignore WS
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "start")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "T")])])]), Tree("term", [Tree("terminal", [Token(TERMINAL, "T")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""a"")])])]), Tree("import", [Tree("terminal", [Token(TERMINAL, "WS")])]), Tree("ignore", [Tree("terminal", [Token(TERMINAL, "WS")])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("non_terminal", [Token(RULE, "t")])]), Tree("non_terminal", [Token(RULE, "t")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "t")]), Tree("string", [Token(STRING, ""a"")])]), Tree("string", [Token(STRING, ""b"")])])])])"#
    );

    make_test_case!(
        test_priority,
        r#"
        s: e
        ?e: t
        t: "hello"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "?e")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "t")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""hello"")])])])])"#
    );

    make_test_case!(
        test_terminal_priority,
        r#"
        start: KEYWORD
        KEYWORD.10: "select"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "start")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "KEYWORD")])])]), Tree("term", [Tree("terminal", [Token(TERMINAL, "KEYWORD")]), Tree("priority", [Token(NUMBER, "10")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""select"")])])])])"#
    );

    make_test_case!(
        test_addition_range,
        r#"
        s: e
        e: e "+" t
        t: "0".."9"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, ""+"")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("range", [Token(STRING, ""0""), Token(STRING, ""9"")])])])])"#
    );

    make_test_case!(
        test_to_add,
        r#"
        s: e
        e: e "+" t -> add
        t: "0".."9"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("alias", [Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, ""+"")]), Tree("non_terminal", [Token(RULE, "t")]), Tree("non_terminal", [Token(RULE, "add")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("range", [Token(STRING, ""0""), Token(STRING, ""9"")])])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("alias", [Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, ""+"")]), Tree("non_terminal", [Token(RULE, "t")]), Tree("non_terminal", [Token(RULE, "add")])]), Tree("alias", [Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, ""-"")]), Tree("non_terminal", [Token(RULE, "t")]), Tree("non_terminal", [Token(RULE, "sub")])]), Tree("non_terminal", [Token(RULE, "t")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("range", [Token(STRING, ""0""), Token(STRING, ""9"")])])])])"#
    );

    make_test_case!(
        test_optional_with_or,
        r#"
        s: e
        e: (e ("+" | "-"))? t
        t: "0".."9"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""+"")]), Tree("string", [Token(STRING, ""-"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("range", [Token(STRING, ""0""), Token(STRING, ""9"")])])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""+"")]), Tree("string", [Token(STRING, ""-"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "INT")])])]), Tree("import", [Tree("terminal", [Token(TERMINAL, "WS")]), Tree("terminal", [Token(TERMINAL, "INT")])]), Tree("ignore", [Tree("terminal", [Token(TERMINAL, "WS")])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""+"")]), Tree("string", [Token(STRING, ""-"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""*"")]), Tree("string", [Token(STRING, ""\"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "d")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "d")]), Tree("or_expansion", [Tree("expansion", [Tree("string", [Token(STRING, ""("")]), Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, "")"")])]), Tree("non_terminal", [Token(RULE, "v")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "v")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "INT")])])]), Tree("import", [Tree("terminal", [Token(TERMINAL, "WS")]), Tree("terminal", [Token(TERMINAL, "INT")])]), Tree("ignore", [Tree("terminal", [Token(TERMINAL, "WS")])])])"#
    );

    #[cfg(feature = "debug")]
    #[test]
    fn load_grammar_returns_error_instead_of_panicking_for_invalid_grammar() {
        let parser_opt = Arc::new(ParserOption::default());
        let result = load_grammar("start T", parser_opt);
        assert!(matches!(result, Err(ParserError::GrammarParseError(_))));
    }

    #[cfg(not(feature = "debug"))]
    #[test]
    fn load_grammar_returns_error_instead_of_panicking_for_invalid_grammar() {
        let result = load_grammar("start T");
        assert!(matches!(result, Err(ParserError::GrammarParseError(_))));
    }
}
