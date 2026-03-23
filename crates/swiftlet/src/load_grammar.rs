use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::builder::GrammarBuilder;
use crate::common::get_common_terminals;
use crate::grammar::{Algorithm, Rule, create_rules};
use crate::lexer::{LexerConf, RegexFlag, Symbol, TerminalDef};
use crate::parser_frontends::ParserConf;
use crate::parser_frontends::ParserFrontend;
use crate::transform::Transformer;
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
        terminal_def!("_COLON", ":"),
        terminal_def!("_OR", r"|"),
        terminal_def!("_DOT", r"\.(?!\.)", RegexFlag::default()),
        terminal_def!("_DOT_DOT", r".."),
        terminal_def!("RULE", r"(_|\?)?[a-z][_a-z0-9]*", RegexFlag::default()),
        terminal_def!("TERMINAL", "_?[A-Z][_A-Z0-9]*", RegexFlag::default()),
        terminal_def!("STRING", r#""(\\"|\\|[^"\n])*?"i?"#, RegexFlag::default()),
        terminal_def!("REGEXP", _regex.as_str(), RegexFlag::default()),
        terminal_def!("_NL_OR", r"(\r?\n)+\s*\|", RegexFlag::default()),
        terminal_def!("_NL", r"(\r?\n)+\s*", RegexFlag::default()),
        terminal_def!("WS", r"[ \t]+", RegexFlag::default()),
        terminal_def!("BACKSLASH", r"\\[ ]*\n", RegexFlag::default()),
        terminal_def!("_TO", "->"),
        terminal_def!("_IGNORE", r"%ignore"),
        terminal_def!("_IMPORT", r"%import"),
        terminal_def!("NUMBER", r"[+-]?\d+", RegexFlag::default()),
        terminal_def!("TILDE", "~", RegexFlag::default()),
        terminal_def!("_COMMA", ",", RegexFlag::default()),
        terminal_def!("_COLON", ":"),
        terminal_def!("_OR", r"|"),
        terminal_def!("_LPAR", "("),
        terminal_def!("_RPAR", ")"),
        terminal_def!("_LBAR", "["),
        terminal_def!("_RBAR", "]"),
        terminal_def!("_LBRACE", "{"),
        terminal_def!("_RBRACE", "}"),
        terminal_def!("OP", r"[+*]|[?](?![a-z_])", RegexFlag::default()),
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

/// Creates the parser frontend used for parsing grammar definitions.
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

/// Parses grammar text and transforms it into a runnable parser frontend.
///
/// Panics if grammar parsing or transformation fails.

macro_rules! impl_load_grammar {
    ($( $extra_arg:ident : $extra_type:ty ),*) => {
        /// Loads a grammar definition into a ready-to-use parser frontend.
        pub fn load_grammar(grammar: &str, $( $extra_arg : $extra_type ),*) -> Arc<ParserFrontend> {
            let tree = match GRAMMAR_BUILDER.parse(grammar) {
                Ok(tree) => tree,
                Err(e) => panic!("Failed to parse grammar. Error: {}", e)
            };
            $(
                if $extra_arg.debug {
                    println!("\nAST of Grammar");
                    println!("==============");
                    tree.pretty_print();
                    println!();
                }
            )*

            let mut transformer = Transformer::new(get_common_terminals());

            transformer.transform(&tree);
            transformer.sort_terminals();

            let rules = transformer.get_grammar();
            $(
                if $extra_arg.debug {
                    transformer.print_terminals();
                    transformer.print_grammar();
                }
            )*

            let parser_conf = Arc::new(ParserConf::new(rules, transformer.get_ignores()));
            let lexer_conf = Arc::new(LexerConf::new(transformer.get_terminal()));

            Arc::new(ParserFrontend::new(lexer_conf, parser_conf))
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "start")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "T")])])]), Tree("term", [Tree("terminal", [Token(TERMINAL, "T")]), Tree("or_expansion", [Tree("string", [Token(STRING, ""a"")])])]), Tree("import", [Tree("terminal", [Token(TERMINAL, "WS")])]), Tree("ignore", [Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "WS")])])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("non_terminal", [Token(RULE, "t")])])]), Tree("non_terminal", [Token(RULE, "t")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "t")]), Tree("string", [Token(STRING, ""a"")])])]), Tree("string", [Token(STRING, ""b"")])])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("or_expansion", [Tree("or_expansion", [Tree("alias", [Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, ""+"")]), Tree("non_terminal", [Token(RULE, "t")]), Tree("non_terminal", [Token(RULE, "add")])])]), Tree("alias", [Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, ""-"")]), Tree("non_terminal", [Token(RULE, "t")]), Tree("non_terminal", [Token(RULE, "sub")])])]), Tree("non_terminal", [Token(RULE, "t")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("range", [Token(STRING, ""0""), Token(STRING, ""9"")])])])])"#
    );

    make_test_case!(
        test_optional_with_or,
        r#"
        s: e
        e: (e ("+" | "-"))? t
        t: "0".."9"
        "#,
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("or_expansion", [Tree("string", [Token(STRING, ""+"")])]), Tree("string", [Token(STRING, ""-"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("range", [Token(STRING, ""0""), Token(STRING, ""9"")])])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("or_expansion", [Tree("string", [Token(STRING, ""+"")])]), Tree("string", [Token(STRING, ""-"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "INT")])])]), Tree("import", [Tree("name_list", [Tree("name_list", [Tree("terminal", [Token(TERMINAL, "WS")])]), Tree("terminal", [Token(TERMINAL, "INT")])])]), Tree("ignore", [Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "WS")])])])])"#
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
        r#"Tree("start", [Tree("rule", [Tree("non_terminal", [Token(RULE, "s")]), Tree("or_expansion", [Tree("non_terminal", [Token(RULE, "e")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "e")]), Tree("or_expansion", [Tree("or_expansion", [Tree("string", [Token(STRING, ""+"")])]), Tree("string", [Token(STRING, ""-"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "t")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("expansion", [Tree("op_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("non_terminal", [Token(RULE, "t")]), Tree("or_expansion", [Tree("or_expansion", [Tree("string", [Token(STRING, ""*"")])]), Tree("string", [Token(STRING, ""\"")])])])]), Token(OP, "?")]), Tree("non_terminal", [Token(RULE, "d")])])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "d")]), Tree("or_expansion", [Tree("or_expansion", [Tree("expansion", [Tree("string", [Token(STRING, ""("")]), Tree("non_terminal", [Token(RULE, "e")]), Tree("string", [Token(STRING, "")"")])])]), Tree("non_terminal", [Token(RULE, "v")])])]), Tree("rule", [Tree("non_terminal", [Token(RULE, "v")]), Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "INT")])])]), Tree("import", [Tree("name_list", [Tree("name_list", [Tree("terminal", [Token(TERMINAL, "WS")])]), Tree("terminal", [Token(TERMINAL, "INT")])])]), Tree("ignore", [Tree("or_expansion", [Tree("terminal", [Token(TERMINAL, "WS")])])])])"#
    );
}
