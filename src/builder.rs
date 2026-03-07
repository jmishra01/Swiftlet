use crate::grammar::Algorithm;
use crate::lexer::{Tokenizer, AST};
use crate::parser::clr::Clr;
use crate::parser::earley::EarleyParser;
use crate::parser::error::ParserError;
use crate::parser::Parser;
use crate::parser_frontends::ParserFrontend;
use crate::ParserOption;
use std::sync::Arc;

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

    /// Tokenizes input text with the provided ignore list.
    pub fn get_tokens(&self, text: &str, ignore: &[String]) -> Tokenizer {
        self.parser.get_parser_frontend().tokenizer(text, ignore)
    }

    /// Parses input text into an AST by tokenizing then invoking the selected parser.
    pub fn parse(&self, text: &str) -> Result<AST, ParserError> {
        let tokens = self.get_tokens(
            text,
            self.parser.get_parser_frontend().get_parser().get_ignores(),
        );
        self.parser.parse(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_grammar::load_grammar;
    use std::sync::Arc;

    fn grammar_text() -> String {
        r#"
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        "#
        .to_string()
    }

    #[test]
    fn builder_get_tokens_and_parse_with_earley() {
        let parser_opt = Arc::new(ParserOption::default());
        let parser_frontend = load_grammar(grammar_text(), parser_opt.clone());
        let builder = GrammarBuilder::new(parser_frontend, parser_opt);

        let mut tokenizer = builder.get_tokens("123", &[]);
        assert_eq!(tokenizer.next().unwrap().word, "123".to_string());
        assert!(builder.parse("123").is_ok());
    }

    #[test]
    fn builder_parse_with_clr() {
        let parser_opt = Arc::new(ParserOption {
            algorithm: Algorithm::CLR,
            ..ParserOption::default()
        });
        let parser_frontend = load_grammar(grammar_text(), parser_opt.clone());
        let builder = GrammarBuilder::new(parser_frontend, parser_opt);

        assert!(builder.parse("42").is_ok());
    }
}
