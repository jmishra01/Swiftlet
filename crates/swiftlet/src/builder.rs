use crate::ParserOption;
use crate::ast::AST;
use crate::error::ParserError;
use crate::grammar::Algorithm;
use crate::lexer::Tokenizer;
use crate::parser::{Parser, clr::Clr, earley::EarleyParser};
use crate::parser_frontends::ParserFrontend;
use std::sync::Arc;

/// Builds and executes the concrete parser selected by [`ParserOption`].
pub struct GrammarBuilder {
    parser: Box<dyn Parser + Send + Sync>,
}

impl GrammarBuilder {
    /// Creates a grammar builder backed by the parser selected in `parser_conf`.
    pub fn new(parser_frontend: Arc<ParserFrontend>, parser_conf: Arc<ParserOption>) -> Self {
        match parser_conf.algorithm {
            Algorithm::Earley => Self {
                parser: Box::new(EarleyParser::new(parser_frontend, parser_conf)),
            },
            Algorithm::CLR => Self {
                parser: Box::new(Clr::new(parser_frontend, parser_conf)),
            },
        }
    }

    /// Tokenizes input text using the parser frontend's cached ignore symbols.
    pub fn get_tokens(&self, text: &str) -> Tokenizer {
        self.parser.get_parser_frontend().tokenizer(text)
    }

    /// Tokenizes input text and materializes the resulting token stream.
    pub fn tokens(&self, text: &str) -> Result<Vec<crate::lexer::Token>, ParserError> {
        let mut tokenizer = self.get_tokens(text);
        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next_token()? {
            tokens.push(token.as_ref().clone());
        }
        Ok(tokens)
    }

    /// Parses input text into an AST by tokenizing then invoking the selected parser.
    pub fn parse(&self, text: &str) -> Result<AST, ParserError> {
        let tokens = self.get_tokens(text);
        self.parser.parse(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_grammar::load_grammar;
    use std::sync::Arc;

    #[cfg(feature = "debug")]
    fn test_frontend(grammar: &str, parser_opt: Arc<ParserOption>) -> Arc<ParserFrontend> {
        load_grammar(grammar, parser_opt).expect("failed to load grammar")
    }

    #[cfg(not(feature = "debug"))]
    fn test_frontend(grammar: &str, _parser_opt: Arc<ParserOption>) -> Arc<ParserFrontend> {
        load_grammar(grammar).expect("failed to load grammar")
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
    fn builder_get_tokens_and_parse_with_earley() {
        let parser_opt = Arc::new(ParserOption::default());
        let parser_frontend = test_frontend(grammar_text(), parser_opt.clone());
        let builder = GrammarBuilder::new(parser_frontend, parser_opt);

        let mut tokenizer = builder.get_tokens("123");
        assert_eq!(tokenizer.next().unwrap().word(), "123");
        assert_eq!(builder.tokens("123").unwrap()[0].word(), "123");
        assert!(builder.parse("123").is_ok());
    }

    #[test]
    fn builder_parse_with_clr() {
        let parser_opt = Arc::new(ParserOption {
            algorithm: Algorithm::CLR,
            ..ParserOption::default()
        });
        let parser_frontend = test_frontend(grammar_text(), parser_opt.clone());
        let builder = GrammarBuilder::new(parser_frontend, parser_opt);

        assert!(builder.parse("42").is_ok());
    }

    #[test]
    fn builder_parse_returns_tokenization_error() {
        let parser_opt = Arc::new(ParserOption::default());
        let parser_frontend = test_frontend(grammar_text(), parser_opt.clone());
        let builder = GrammarBuilder::new(parser_frontend, parser_opt);

        let err = builder.parse("abc").expect_err("tokenization should fail");
        assert!(matches!(err, ParserError::TokenizationError { .. }));
    }
}
