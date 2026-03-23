use crate::grammar::{Rule, RuleOption};
use crate::lexer::{Symbol, Token, Tokenizer};
use crate::ast::AST;
use crate::parser::Parser;
use crate::parser::error::ParserError;
use crate::parser::utils::dot_state;
use crate::parser_frontends::ParserFrontend;
use crate::{Ambiguity, ParserOption, non_terms};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::Iterator;
use std::sync::Arc;

/// Represents a single Earley item together with accumulated children.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub rule: Arc<Rule>,
    pub dot: usize,
    pub start: usize,
    pub end: usize,
    pub children: Vec<AST>,
}

/// Deduplication key for Earley states that ignores child trees.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StateCore {
    rule: Arc<Rule>,
    dot: usize,
    start: usize,
    end: usize,
}

/// Stores all Earley states generated for a single chart position.
#[derive(Default)]
struct ChartColumn {
    states: Vec<Arc<State>>,
    exact_index: HashMap<StateCore, Vec<Arc<State>>>,
    pending_by_symbol: HashMap<Arc<Symbol>, Vec<Arc<State>>>,
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

impl ChartColumn {
    /// Inserts a state if it has not already been seen in this column.
    fn insert(&mut self, state: Arc<State>) -> Option<usize> {
        let core = StateCore {
            rule: state.rule.clone(),
            dot: state.dot,
            start: state.start,
            end: state.end,
        };

        if let Some(existing) = self.exact_index.get(&core)
            && existing
                .iter()
                .any(|candidate| candidate.as_ref() == state.as_ref())
        {
            return None;
        }

        let index = self.states.len();
        self.states.push(state.clone());
        self.exact_index
            .entry(core)
            .or_default()
            .push(state.clone());
        if let Some(next_symbol) = state.next_symbol()
            && !next_symbol.is_terminal()
        {
            self.pending_by_symbol
                .entry(next_symbol)
                .or_default()
                .push(state);
        }
        Some(index)
    }
}

impl Display for State {
    /// Formats state as `A -> alpha ● beta`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (rule, before_dot, after_dot) = dot_state(&self.rule, self.dot);
        write!(f, "{rule} -> {before_dot} ● {after_dot}")
    }
}

/// Earley parser implementation used for general context-free grammars.
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
        chart: &mut [ChartColumn],
        worklist: &mut Vec<usize>,
        next_symbol: Arc<Symbol>,
        i: usize,
    ) {
        let mut pending_states = Vec::new();
        for rule in self
            .parser_frontend
            .get_parser()
            .next_expansion(&next_symbol)
        {
            pending_states.push(Arc::new(State {
                rule: rule.clone(),
                dot: 0,
                start: i,
                end: i,
                children: vec![],
            }));
        }

        for next_state in pending_states {
            if let Some(index) = chart[i].insert(next_state) {
                worklist.push(index);
            }
        }
    }

    /// Earley completion step: advances states waiting on a completed non-terminal.
    #[inline(always)]
    fn complete(
        &self,
        chart: &mut [ChartColumn],
        worklist: &mut Vec<usize>,
        state: Arc<State>,
        i: usize,
    ) {
        let candidates = chart[state.start]
            .pending_by_symbol
            .get(&state.rule.origin)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let mut pending_states = Vec::with_capacity(candidates.len());
        for x1 in candidates {
            let mut child = Vec::with_capacity(x1.children.len() + state.children.len() + 1);
            child.extend(x1.children.iter().cloned());
            if state.rule.origin.starts_with("_")
                || (x1.rule.is_expand() && x1.rule.origin == state.rule.origin)
            {
                child.extend(state.children.iter().cloned());
            } else if state.children.len() == 1 && state.rule.is_expand() {
                child.push(state.children[0].clone());
            } else if state.children.len() == 1
                && let Some(AST::Tree(name, _)) = state.children.first()
                && let Some(alias_rule) = state.rule.rule_option.alias_rule()
                && alias_rule.contains(name)
            {
                child.push(state.children[0].clone());
            } else {
                child.push(AST::Tree(
                    state.rule.origin.as_ref().as_str().to_string(),
                    state.children.clone(),
                ));
            }

            pending_states.push(Arc::new(State {
                rule: x1.rule.clone(),
                dot: x1.dot + 1,
                start: x1.start,
                end: i,
                children: child,
            }));
        }

        for next_state in pending_states {
            if let Some(index) = chart[i].insert(next_state) {
                worklist.push(index);
            }
        }
    }

    /// Earley scan step: consumes a matching terminal token into the next chart column.
    #[inline(always)]
    fn scan(
        &self,
        chart: &mut Vec<ChartColumn>,
        token: Option<Arc<Token>>,
        state: &Arc<State>,
        next_symbol: Arc<Symbol>,
        i: usize,
    ) {
        if let Some(token) = token.clone()
            && next_symbol == token.terminal
        {
            let mut child = Vec::with_capacity(state.children.len() + 1);
            child.extend(state.children.iter().cloned());
            if !token.terminal.starts_with("_") || token.terminal.starts_with("__") {
                child.push(AST::Token(token.clone()));
            }

            if chart.get(i + 1).is_none() {
                chart.push(ChartColumn::default());
            }

            let next_state = Arc::new(State {
                rule: state.rule.clone(),
                dot: state.dot + 1,
                start: state.start,
                end: i + 1,
                children: child,
            });

            let _ = chart[i + 1].insert(next_state);
        }
    }
}

impl Parser for EarleyParser {
    /// Returns parser frontend.
    fn get_parser_frontend(&self) -> Arc<ParserFrontend> {
        self.parser_frontend.clone()
    }

    /// Runs Earley parsing and returns an AST according to ambiguity strategy.
    fn parse(&self, mut token_iter: Tokenizer) -> Result<AST, ParserError> {
        let mut chart = vec![ChartColumn::default()];

        let start_rule = Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal("gamma".to_string())),
            vec![non_terms!(self.parser_config.start)],
            Arc::new(RuleOption::default()),
            0,
        ));

        let _ = chart[0].insert(Arc::new(State::new(start_rule, 0, 0, 0, vec![])));
        let mut j = 1;
        let mut i = 0;

        #[cfg(feature = "debug")]
        if self.parser_config.debug {
            println!("\nEarley Parser");
            println!("=============");
        }

        while i <= j {
            let token = token_iter.next();

            if token.is_some() {
                j += 1;
            }
            if chart.get(i).is_none() {
                chart.push(ChartColumn::default());
            }

            let mut worklist = (0..chart[i].states.len()).collect::<Vec<_>>();
            while let Some(state_index) = worklist.pop() {
                let state = chart[i].states[state_index].clone();
                if state.is_complete() {
                    self.complete(&mut chart, &mut worklist, state, i);
                } else if let Some(next_symbol) = state.next_symbol() {
                    if self
                        .parser_frontend
                        .get_parser()
                        .contains_rule(&next_symbol)
                    {
                        self.prediction(&mut chart, &mut worklist, next_symbol, i);
                    } else {
                        self.scan(&mut chart, token.clone(), &state, next_symbol, i);
                    }
                }
            }

            #[cfg(feature = "debug")]
            if self.parser_config.debug {
                println!("Index: {} | {:?}", i, token);
                for state in chart[i].states.iter() {
                    println!("\tState: {}", state);
                }
            }
            i += 1;
        }

        chart.remove(chart.len() - 1);

        let mut complete_parsed_tree = chart
            .last()
            .unwrap()
            .states
            .iter()
            .filter(|x| x.rule.origin.as_ref().as_str() == "gamma");

        match self.parser_config.ambiguity {
            Ambiguity::Resolve => {
                if let Some(states) = complete_parsed_tree.next()
                    && let Some(children) = states.children.first()
                {
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
        Err(ParserError::FailedToParse(
            token_iter.get_text().to_string(),
        ))
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
        let pf = load_grammar(grammar);
        let earley = EarleyParser::new(pf.clone(), parser_opt);
        let tk = pf.tokenizer("x");
        assert!(earley.parse(tk).is_ok());

        let explicit_opt = Arc::new(ParserOption {
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Explicit,
            ..ParserOption::default()
        });
        let explicit_pf = load_grammar(grammar);
        let explicit = EarleyParser::new(explicit_pf.clone(), explicit_opt);
        let ast = explicit.parse(explicit_pf.tokenizer("x")).unwrap();
        assert_eq!(ast.get_tree_name(), Some(&"_ambiguity".to_string()));
    }
}
