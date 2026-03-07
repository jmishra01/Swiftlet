use crate::grammar::{Rule, RuleOption};
use crate::lexer::{Symbol, Token, Tokenizer, AST};
use crate::parser::error::ParserError;
use crate::parser::utils::dot_state;
use crate::parser::Parser;
use crate::parser_frontends::ParserFrontend;
use crate::{non_terms, Ambiguity, ParserOption};
use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::Iterator;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub rule: Arc<Rule>,
    pub dot: usize,
    pub start: usize,
    pub end: usize,
    pub children: Vec<AST>,
}

impl Hash for State {
    /// Hashes state identity fields used for chart deduplication.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl State {
    /// Creates an Earley state.
    pub fn new(rule: Arc<Rule>, dot: usize, start: usize, end: usize, children: Vec<AST>) -> Self {
        Self {
            rule,
            dot,
            start,
            end,
            children,
        }
    }

    /// Returns whether the state has consumed the full rule expansion.
    pub fn is_complete(&self) -> bool {
        self.dot == self.rule.len()
    }

    /// Returns the next expected symbol, if any.
    pub fn next_symbol(&self) -> Option<Arc<Symbol>> {
        if self.dot < self.rule.len() {
            return Some(self.rule.expansion[self.dot].clone());
        }
        None
    }
}

impl Display for State {
    /// Formats state as `A -> alpha ● beta`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (rule, before_dot, after_dot) = dot_state(&self.rule, self.dot);
        write!(f, "{rule} -> {before_dot} ● {after_dot}")
    }
}

pub struct EarleyParser {
    parser_frontend: Arc<ParserFrontend>,
    parser_config: Arc<ParserOption>,
}

impl EarleyParser {
    /// Creates an Earley parser.
    pub fn new(parser_frontend: Arc<ParserFrontend>, parser_config: Arc<ParserOption>) -> Self {
        Self {
            parser_frontend,
            parser_config,
        }
    }

    /// Earley prediction step: adds states for rules of the expected non-terminal.
    #[inline(always)]
    fn prediction(
        &self,
        chart: &mut [HashSet<Arc<State>>],
        worklist: &mut VecDeque<Arc<State>>,
        next_symbol: Arc<Symbol>,
        i: usize,
        added: bool,
    ) -> bool {
        // PREDICTION
        let mut added = added;
        for rule in self
            .parser_frontend
            .get_parser()
            .next_expansion(&next_symbol)
        {
            let next_state = Arc::new(State {
                rule: rule.clone(),
                dot: 0,
                start: i,
                end: i,
                children: vec![],
            });

            if chart[i].insert(next_state.clone()) {
                worklist.push_back(next_state);
                added = true;
            }
        }
        added
    }

    /// Earley completion step: advances states waiting on a completed non-terminal.
    #[inline(always)]
    fn complete(
        &self,
        chart: &mut [HashSet<Arc<State>>],
        worklist: &mut VecDeque<Arc<State>>,
        state: Arc<State>,
        i: usize,
        added: bool,
    ) -> bool {
        // COMPLETE
        let mut added = added;
        let candidates: Vec<Arc<State>> = chart[state.start]
            .iter()
            .filter_map(|x| {
                if let Some(next_symbol) = x.next_symbol()
                    && next_symbol == state.rule.origin
                {
                    return Some(x.clone());
                }
                None
            })
            .collect();

        for x1 in candidates {
            let mut child = x1.children.clone();
            if state.rule.origin.starts_with("_")
                || (x1.rule.is_expand() && x1.rule.origin == state.rule.origin)
            {
                for ast in state.children.iter() {
                    child.push(ast.clone());
                }
            } else if state.rule.is_expand() && state.children.len() == 1 {
                child.push(state.children[0].clone());
            } else {
                child.push(AST::Tree(
                    state.rule.origin.as_ref().as_str().to_string(),
                    state.children.clone(),
                ));
            }

            let next_state = Arc::new(State {
                rule: x1.rule.clone(),
                dot: x1.dot + 1,
                start: x1.start,
                end: i,
                children: child,
            });

            if chart[i].insert(next_state.clone()) {
                worklist.push_back(next_state);
                added = true;
            }
        }

        added
    }

    /// Earley scan step: consumes a matching terminal token into the next chart column.
    #[inline(always)]
    fn scan(
        &self,
        chart: &mut Vec<HashSet<Arc<State>>>,
        token: Option<Arc<Token>>,
        state: &Arc<State>,
        next_symbol: Arc<Symbol>,
        i: usize,
        added: bool,
    ) -> bool {
        // SCAN
        let mut added = added;
        if let Some(token) = token.clone()
            && next_symbol == token.terminal
        {
            let mut child = state.children.clone();
            if !token.terminal.starts_with("_") || token.terminal.starts_with("__") {
                child.push(AST::Token(token.clone()));
            }

            if chart.get(i + 1).is_none() {
                chart.push(HashSet::new());
            }

            let next_state = Arc::new(State {
                rule: state.rule.clone(),
                dot: state.dot + 1,
                start: state.start,
                end: i + 1,
                children: child,
            });

            if chart[i + 1].insert(next_state) {
                added = true;
            }
        }
        added
    }
}

impl Parser for EarleyParser {
    /// Returns parser frontend.
    fn get_parser_frontend(&self) -> Arc<ParserFrontend> {
        self.parser_frontend.clone()
    }

    /// Runs Earley parsing and returns an AST according to ambiguity strategy.
    fn parse(&self, mut token_iter: Tokenizer) -> Result<AST, ParserError> {
        let mut chart = vec![];

        chart.push(HashSet::new());

        let start_rule = Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal("gamma".to_string())),
            vec![non_terms!(self.parser_config.start)],
            Arc::new(RuleOption::default()),
            0,
        ));

        chart[0].insert(Arc::new(State::new(start_rule, 0, 0, 0, vec![])));
        let mut j = 1;
        let mut i = 0;

        while i <= j {
            let token = token_iter.next();

            if token.is_some() {
                j += 1;
            }
            if chart.get(i).is_none() {
                chart.push(HashSet::new());
            }

            let mut worklist: VecDeque<Arc<State>> = chart[i].iter().cloned().collect();
            while let Some(state) = worklist.pop_front() {
                if state.is_complete() {
                    // COMPLETE
                    self.complete(&mut chart, &mut worklist, state, i, false);
                } else if let Some(next_symbol) = state.next_symbol() {
                    if self
                        .parser_frontend
                        .get_parser()
                        .contains_rule(&next_symbol)
                    {
                        // PREDICTION
                        self.prediction(&mut chart, &mut worklist, next_symbol, i, false);
                    } else {
                        // SCAN
                        self.scan(&mut chart, token.clone(), &state, next_symbol, i, false);
                    }
                }
            }
            if self.parser_config.debug {
                println!("Index: {} | {:?}", i, token);
                for state in chart[i].iter() {
                    println!("\tState: {}", state);
                }
            }
            i += 1;
        }

        chart.remove(chart.len() - 1);

        let mut complete_parsed_tree = chart
            .last()
            .unwrap()
            .iter()
            .filter(|&x| x.rule.origin.as_ref().as_str() == "gamma");

        match self.parser_config.ambiguity {
            Ambiguity::Resolve => {
                if let Some(states) = complete_parsed_tree.next()
                    && let Some(children) = states.children.first() {
                    return Ok(children.clone());
                }
            }
            Ambiguity::Explicit => {
                let mut children = Vec::new();
                for states in complete_parsed_tree {
                    children.push(states.children.first().cloned().unwrap());
                }

                return Ok(AST::Tree("_ambiguity".to_string(), children));
            }
        }
        Err(ParserError::FailedToParse(token_iter.get_text().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::Algorithm;
    use crate::load_grammar::load_grammar;

    #[test]
    fn state_core_methods_and_display_work() {
        let rule = Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal("expr".to_string())),
            vec![
                Arc::new(Symbol::NonTerminal("expr".to_string())),
                Arc::new(Symbol::Terminal("INT".to_string())),
            ],
            Arc::new(RuleOption::default()),
            0,
        ));
        let s0 = State::new(rule.clone(), 0, 0, 0, vec![]);
        let s2 = State::new(rule, 2, 0, 1, vec![]);

        assert!(!s0.is_complete());
        assert_eq!(s0.next_symbol().unwrap().as_ref().as_str(), "expr");
        assert!(s2.is_complete());
        assert!(s2.next_symbol().is_none());
        assert!(format!("{s0}").contains("expr ->"));
    }

    #[test]
    fn earley_parser_parses_and_explicit_ambiguity_returns_tree() {
        let grammar = r#"
        start: a
        a: "x" | "x"
        "#;
        let parser_opt = Arc::new(ParserOption::default());
        let pf = load_grammar(grammar.to_string(), parser_opt.clone());
        let earley = EarleyParser::new(pf.clone(), parser_opt);
        let tk = pf.tokenizer("x", &[]);
        assert!(earley.parse(tk).is_ok());

        let explicit_opt = Arc::new(ParserOption {
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Explicit,
            ..ParserOption::default()
        });
        let explicit_pf = load_grammar(grammar.to_string(), explicit_opt.clone());
        let explicit = EarleyParser::new(explicit_pf.clone(), explicit_opt);
        let ast = explicit.parse(explicit_pf.tokenizer("x", &[])).unwrap();
        assert_eq!(ast.get_tree_name(), Some(&"_ambiguity".to_string()));
    }
}
