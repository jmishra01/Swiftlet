use crate::grammar::Rule;
use crate::lexer::{LexerConf, Symbol, TerminalDef, Tokenizer};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Stores grammar rules, ignore directives, and cached rule expansions.
#[derive(Debug)]
pub struct ParserConf {
    rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>,
    ignore_symbols: Arc<HashSet<Arc<Symbol>>>,
    all_expansions: Vec<Arc<Rule>>,
}

impl ParserConf {
    /// Creates parser configuration with rule table and ignored terminal names.
    pub fn new(rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>, ignores: Vec<String>) -> Self {
        let ignore_symbols = ignores
            .iter()
            .map(|name| Arc::new(Symbol::Terminal(name.clone())))
            .collect::<HashSet<_>>();
        let all_expansions = rules.values().flatten().cloned().collect();
        Self {
            rules,
            ignore_symbols: Arc::new(ignore_symbols),
            all_expansions,
        }
    }

    /// Returns cached ignored terminal symbols.
    pub(crate) fn get_ignore_symbols(&self) -> &Arc<HashSet<Arc<Symbol>>> {
        &self.ignore_symbols
    }

    /// Checks whether a non-terminal has at least one rule.
    pub fn contains_rule(&self, name: &Arc<Symbol>) -> bool {
        self.rules.contains_key(name)
    }

    /// Appends a rule to the rule table under its origin symbol.
    pub fn add_rules(&mut self, rule: Arc<Rule>) {
        let val = self.rules.entry(rule.origin.clone()).or_default();
        val.push(rule.clone());
        self.all_expansions.push(rule);
    }

    /// Returns an iterator overrule expansions of a symbol.
    ///
    /// Panics if the symbol does not exist in the rule map.
    pub fn next_expansion(&self, name: &Arc<Symbol>) -> impl Iterator<Item = &Arc<Rule>> + '_ {
        self.rules.get(name).unwrap().iter()
    }

    /// Returns all rule expansions from the grammar.
    pub fn get_all_expansion(&self) -> &[Arc<Rule>] {
        &self.all_expansions
    }

    /// Returns all rules for the specified symbol, if present.
    pub fn get_expansion(&self, key: &Arc<Symbol>) -> Option<&Vec<Arc<Rule>>> {
        self.rules.get(key)
    }
}

/// Bundles lexer and parser configuration for a compiled grammar.
#[derive(Debug)]
pub struct ParserFrontend {
    lexer: Arc<LexerConf>,
    parser: Arc<ParserConf>,
    ignore_terminals: Arc<[Arc<TerminalDef>]>,
}

impl ParserFrontend {
    /// Creates a parser frontend with lexer and parser configurations.
    pub(crate) fn new(lexer: Arc<LexerConf>, parser: Arc<ParserConf>) -> Self {
        let ignore_terminals = Arc::from(
            parser
                .get_ignore_symbols()
                .iter()
                .filter_map(|symbol| lexer.get_terminal_def(symbol).cloned())
                .collect::<Vec<_>>(),
        );

        Self {
            lexer,
            parser,
            ignore_terminals,
        }
    }

    /// Returns a tokenizer for `text` using cached ignored terminal symbols.
    pub(crate) fn tokenizer(&self, text: &str) -> Tokenizer {
        self.lexer.tokenize(
            text,
            self.parser.get_ignore_symbols().clone(),
            self.ignore_terminals.clone(),
        )
    }

    /// Returns the lexer configuration.
    #[allow(dead_code)]
    pub(crate) fn get_lexer(&self) -> Arc<LexerConf> {
        self.lexer.clone()
    }

    /// Returns the parser configuration.
    pub fn get_parser(&self) -> &Arc<ParserConf> {
        &self.parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::RuleOption;

    fn sample_rule(origin: &str, expansion: &[&str]) -> Arc<Rule> {
        Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal(origin.to_string())),
            expansion
                .iter()
                .map(|x| crate::lexer::get_symbol(x))
                .collect(),
            Arc::new(RuleOption::default()),
            0,
        ))
    }

    #[test]
    fn parser_conf_crud_and_iteration_work() {
        let start_rule = sample_rule("start", &["expr"]);
        let mut pc = ParserConf::new(
            HashMap::from([(
                Arc::new(Symbol::NonTerminal("start".to_string())),
                vec![start_rule],
            )]),
            vec!["WS".to_string()],
        );

        let expr_rule = sample_rule("expr", &["INT"]);
        pc.add_rules(expr_rule.clone());

        let start = Arc::new(Symbol::NonTerminal("start".to_string()));
        let expr = Arc::new(Symbol::NonTerminal("expr".to_string()));

        assert!(pc.contains_rule(&start));
        assert!(pc.contains_rule(&expr));
        assert!(
            pc.get_ignore_symbols()
                .contains(&Arc::new(Symbol::Terminal("WS".to_string())))
        );
        assert_eq!(pc.next_expansion(&expr).count(), 1);
        assert_eq!(pc.get_all_expansion().len(), 2);
        assert_eq!(pc.get_expansion(&expr).unwrap().len(), 1);
    }
}
