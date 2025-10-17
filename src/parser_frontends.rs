use crate::grammar::Rule;
use crate::lexer::{LexerConf, Symbol, Tokenizer};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct ParserConf {
    rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>,
    ignores: Vec<String>,
}

impl ParserConf {
    pub fn new(rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>, ignores: Vec<String>) -> Self {
        Self { rules, ignores }
    }

    pub(crate) fn get_ignores(&self) -> &Vec<String> {
        &self.ignores
    }

    pub fn contains_rule(&self, name: &Arc<Symbol>) -> bool {
        self.rules.contains_key(name)
    }

    pub fn add_rules(&mut self, rule: Arc<Rule>) {
        let val = self.rules.entry(rule.origin.clone()).or_default();
        val.push(rule);
    }

    pub fn next_expansion(&self, name: &Arc<Symbol>) -> impl Iterator<Item = &Arc<Rule>> + '_ {
        self.rules.get(name).unwrap().iter()
    }

    pub fn get_all_expansion(&self) -> Vec<Arc<Rule>> {
        #[cfg(feature = "debug")]
        {
            let mut rules = self
                .rules
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<Arc<Rule>>>();
            rules.sort_by_key(|x| x.origin.get_value().clone());
            rules
        }
        #[cfg(not(feature = "debug"))]
        {
            self.rules
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<Arc<Rule>>>()
        }
    }

    pub fn get_expansion(&self, key: &Arc<Symbol>) -> Option<&Vec<Arc<Rule>>> {
        self.rules.get(key)
    }
}

#[derive(Debug)]
pub struct ParserFrontend {
    lexer: Arc<LexerConf>,
    parser: Arc<ParserConf>,
}

impl ParserFrontend {
    pub(crate) fn new(lexer: Arc<LexerConf>, parser: Arc<ParserConf>) -> Self {
        Self { lexer, parser }
    }

    pub(crate) fn tokenizer(&self, text: &str, ignore: &[String]) -> Tokenizer {
        self.lexer.tokenize(text, ignore)
    }

    #[allow(dead_code)]
    pub(crate) fn get_lexer(&self) -> Arc<LexerConf> {
        self.lexer.clone()
    }

    pub fn get_parser(&self) -> &Arc<ParserConf> {
        &self.parser
    }
}
