use crate::grammar::{Rule, RuleOption};
use crate::lexer::{Symbol, Token, Tokenizer, AST};
use crate::parser::error::ParserError;
use crate::parser::utils::dot_state;
use crate::parser::Parser;
use crate::parser_frontends::ParserFrontend;
use crate::{non_terms, Ambiguity, ParserOption};
use std::collections::HashSet;
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl State {
    pub fn new(rule: Arc<Rule>, dot: usize, start: usize, end: usize, children: Vec<AST>) -> Self {
        Self {
            rule,
            dot,
            start,
            end,
            children,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.dot == self.rule.len()
    }

    pub fn next_symbol(&self) -> Option<Arc<Symbol>> {
        if self.dot < self.rule.len() {
            return Some(self.rule.expansion[self.dot].clone());
        }
        None
    }
}

impl Display for State {
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
    pub fn new(parser_frontend: Arc<ParserFrontend>, parser_config: Arc<ParserOption>) -> Self {
        Self {
            parser_frontend,
            parser_config,
        }
    }

    #[inline(always)]
    fn prediction(
        &self,
        chart: &mut [HashSet<Arc<State>>],
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

            if chart[i].insert(next_state) {
                added = true;
            }
        }
        added
    }

    #[inline(always)]
    fn complete(
        &self,
        chart: &mut [HashSet<Arc<State>>],
        state: Arc<State>,
        i: usize,
        added: bool,
    ) -> bool {
        // COMPLETE
        let mut added = added;
        let prev_len = chart[i].len();

        let (left_chart, right_chart) = chart.split_at_mut(i);

        for s in left_chart[state.start]
            .iter()
            .filter(|&x| {
                if let Some(next_symbol) = x.next_symbol() {
                    return next_symbol == state.rule.origin;
                }
                false
            })
            .map(|x1| {
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
                        state.rule.origin.get_value(),
                        state.children.clone(),
                    ));
                }
                Arc::new(State {
                    rule: x1.rule.clone(),
                    dot: x1.dot + 1,
                    start: x1.start,
                    end: i,
                    children: child,
                })
            })
        {
            right_chart[0].insert(s);
        }

        if chart[i].len() > prev_len {
            added = true;
        }

        added
    }

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
    fn get_parser_frontend(&self) -> Arc<ParserFrontend> {
        self.parser_frontend.clone()
    }

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
            let mut cache: HashSet<Arc<State>> = HashSet::new();

            let mut added = true;
            while added {
                added = false;

                if chart.get(i).is_none() {
                    chart.push(HashSet::new());
                }

                let states: HashSet<Arc<State>> = chart[i].clone();

                for state in states {
                    if !cache.insert(state.clone()) {
                        continue;
                    }

                    if state.is_complete() {
                        // COMPLETE
                        added = self.complete(&mut chart, state, i, added);
                    } else if let Some(next_symbol) = state.next_symbol() {
                        if self
                            .parser_frontend
                            .get_parser()
                            .contains_rule(&next_symbol)
                        {
                            // PREDICTION
                            added = self.prediction(&mut chart, next_symbol, i, added);
                        } else {
                            // SCAN
                            added =
                                self.scan(&mut chart, token.clone(), &state, next_symbol, i, added);
                        }
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
            .filter(|&x| x.rule.origin.get_value() == "gamma");

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
