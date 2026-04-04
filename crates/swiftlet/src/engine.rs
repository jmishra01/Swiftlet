use crate::ParserConfig;
use crate::ast::Ast;
use crate::error::SwiftletError;
use crate::grammar::Algorithm;
use crate::lexer::Tokenizer;
use crate::parser::{ParserBackend, clr::ClrParser, earley::EarleyParser};
use crate::parser_frontends::GrammarRuntime;
use std::sync::Arc;

/// Builds and executes the concrete parser selected by [`ParserConfig`].
pub struct ParserEngine {
    parser: Box<dyn ParserBackend + Send + Sync>,
}

impl ParserEngine {
    /// Creates a parser engine backed by the parser selected in `parser_conf`.
    pub fn new(parser_frontend: Arc<GrammarRuntime>, parser_conf: Arc<ParserConfig>) -> Self {
        match parser_conf.algorithm {
            Algorithm::Earley => Self {
                parser: Box::new(EarleyParser::new(parser_frontend, parser_conf)),
            },
            Algorithm::CLR => Self {
                parser: Box::new(ClrParser::new(parser_frontend, parser_conf)),
            },
        }
    }

    /// Tokenizes input text using the parser frontend's cached ignore symbols.
    pub fn tokenize(&self, text: &str) -> Tokenizer {
        self.parser.get_parser_frontend().tokenizer(text)
    }

    /// Parses input text into an AST by tokenizing then invoking the selected parser.
    pub fn parse(&self, text: &str) -> Result<Ast, SwiftletError> {
        let mut tokens = self.tokenize(text);
        self.parser.parse(&mut tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LexerError;
    use crate::load_grammar::load_grammar;
    use std::sync::Arc;

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

    #[cfg(feature = "debug")]
    fn test_frontend(grammar: &str, parser_opt: Arc<ParserConfig>) -> Arc<GrammarRuntime> {
        load_grammar(&normalize_grammar(grammar), parser_opt).expect("failed to load grammar")
    }

    #[cfg(not(feature = "debug"))]
    fn test_frontend(grammar: &str, _parser_opt: Arc<ParserConfig>) -> Arc<GrammarRuntime> {
        load_grammar(&normalize_grammar(grammar)).expect("failed to load grammar")
    }

    fn grammar_text() -> &'static str {
        r#"
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        "#
    }

    #[test]
    fn builder_parse_with_clr() {
        let parser_opt = Arc::new(ParserConfig {
            algorithm: Algorithm::CLR,
            ..ParserConfig::default()
        });
        let parser_frontend = test_frontend(grammar_text(), parser_opt.clone());
        let builder = ParserEngine::new(parser_frontend, parser_opt);

        assert!(builder.parse("42").is_ok());
    }

    #[test]
    fn builder_parse_returns_tokenization_error() {
        let parser_opt = Arc::new(ParserConfig::default());
        let parser_frontend = test_frontend(grammar_text(), parser_opt.clone());
        let builder = ParserEngine::new(parser_frontend, parser_opt);

        let err = builder.parse("abc").expect_err("tokenization should fail");
        assert!(matches!(
            err,
            SwiftletError::Lexer(LexerError::Tokenization { .. })
        ));
    }
}
