use crate::ast::Ast;
use crate::error::{LexerError, ParseError, SwiftletError};
use crate::grammar::{Rule, RuleMeta};
use crate::lexer::{Symbol, Token, TokenMatch, Tokenizer};
use crate::parser::ParserBackend;
use crate::parser::utils::dot_state;
use crate::parser_frontends::GrammarRuntime;
use crate::{Ambiguity, ParserConfig, non_terms};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::Iterator;
use std::sync::Arc;

/// Represents a single Earley item together with accumulated children.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EarleyItem {
    pub rule: Arc<Rule>,
    pub dot: usize,
    pub start: usize,
    pub end: usize,
    pub children: Vec<Ast>,
}

pub(crate) struct SymbolTokenState {
    symbol: Arc<Symbol>,
    token_match: TokenMatch,
    state_index: usize,
    priority: usize,
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
#[derive(Clone, Default)]
struct ChartColumn {
    states: Vec<Arc<EarleyItem>>,
    exact_index: HashMap<StateCore, Vec<Arc<EarleyItem>>>,
    pending_by_symbol: HashMap<Arc<Symbol>, Vec<Arc<EarleyItem>>>,
}

impl EarleyItem {
    /// Creates an Earley state.
    pub fn new(rule: Arc<Rule>, dot: usize, start: usize, end: usize, children: Vec<Ast>) -> Self {
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
    fn insert(&mut self, state: Arc<EarleyItem>) -> Option<usize> {
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

impl Display for EarleyItem {
    /// Formats state as `A -> alpha ● beta`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (rule, before_dot, after_dot) = dot_state(&self.rule, self.dot);
        write!(f, "{rule} -> {before_dot} ● {after_dot}")
    }
}

/// Earley parser implementation used for general context-free grammars.
pub struct EarleyParser {
    parser_frontend: Arc<GrammarRuntime>,
    parser_config: Arc<ParserConfig>,
}

impl EarleyParser {
    /// Creates an Earley parser.
    pub fn new(parser_frontend: Arc<GrammarRuntime>, parser_config: Arc<ParserConfig>) -> Self {
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
            pending_states.push(Arc::new(EarleyItem {
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
        state: Arc<EarleyItem>,
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
                && let Some(Ast::Tree(name, _)) = state.children.first()
                && let Some(alias_rule) = state.rule.rule_option.alias_rule()
                && alias_rule.contains(name)
            {
                child.push(state.children[0].clone());
            } else {
                child.push(Ast::Tree(
                    state.rule.origin.as_ref().as_str().to_string(),
                    state.children.clone(),
                ));
            }

            pending_states.push(Arc::new(EarleyItem {
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
    fn scan(&self, chart: &mut Vec<ChartColumn>, token: Arc<Token>, state: &Arc<EarleyItem>, i: usize) {
        let mut child = Vec::with_capacity(state.children.len() + 1);
        child.extend(state.children.iter().cloned());

        if !token.terminal.starts_with("_") || token.terminal.starts_with("__") {
            child.push(Ast::Token(token.clone()));
        }

        let next_state = Arc::new(EarleyItem {
            rule: state.rule.clone(),
            dot: state.dot + 1,
            start: state.start,
            end: i + 1,
            children: child,
        });

        if chart.get(i + 1).is_none() {
            chart.push(ChartColumn::default());
        }

        let _ = chart[i + 1].insert(next_state);
    }

    fn finalize_basic_parse(
        &self,
        chart: &[ChartColumn],
        tokenizer: &mut Tokenizer,
        expected_token: &[Arc<Symbol>],
    ) -> Result<Ast, SwiftletError> {
        let Some(last_column) = chart.last() else {
            return Err(ParseError::FailedToParse(
                "earley parser produced no chart columns".to_string(),
            )
            .into());
        };

        let mut complete_parsed_tree = last_column
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
                    let Some(child) = states.children.first() else {
                        return Err(ParseError::FailedToParse(
                            "completed parse state did not contain a child AST".to_string(),
                        )
                        .into());
                    };
                    children.push(child.clone());
                }

                if !children.is_empty() {
                    return Ok(Ast::Tree("_ambiguity".to_string(), children));
                }
            }
        }

        let exp = expected_token
            .iter()
            .map(|x| {
                let t = tokenizer.get_terminal_def(x).unwrap();
                t.value.clone()
            })
            .collect::<Vec<_>>();
        let (line, column) = tokenizer.get_line_column();
        Err(LexerError::Tokenization {
            location: tokenizer.get_start(),
            line,
            column,
            expected: exp,
            text: tokenizer.get_text().to_string(),
            caret: format!("{}^", " ".repeat(column - 1)),
        }
        .into())
    }
}

impl ParserBackend for EarleyParser {
    /// Returns parser frontend.
    fn get_parser_frontend(&self) -> &Arc<GrammarRuntime> {
        &self.parser_frontend
    }

    /// Runs Earley parsing and returns an AST according to ambiguity strategy.
    fn parse(&self, token_iter: &mut Tokenizer) -> Result<Ast, SwiftletError> {
        let mut chart = vec![ChartColumn::default()];

        let start_rule = Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal("gamma".to_string())),
            vec![non_terms!(self.parser_config.start)],
            Arc::new(RuleMeta::default()),
            0,
        ));

        let _ = chart[0].insert(Arc::new(EarleyItem::new(start_rule, 0, 0, 0, vec![])));
        let mut j = 1;
        let mut i = 0;

        #[cfg(feature = "debug")]
        if self.parser_config.debug {
            println!("\nEarley Parser");
            println!("=============");
        }

        let mut next_possible_symbols: Vec<SymbolTokenState> = Vec::new();
        let mut prev_next_symbol: Vec<Arc<Symbol>> = Vec::new();

        while i <= j {
            if chart.get(i).is_none() {
                chart.push(ChartColumn::default());
            }

            if !next_possible_symbols.is_empty() {
                prev_next_symbol.clear();
                prev_next_symbol.extend(
                    next_possible_symbols
                        .iter()
                        .map(|candidate| candidate.symbol.clone()),
                );
            }

            next_possible_symbols.clear();

            #[cfg(feature = "debug")]
            let mut token_arr = vec![];

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
                        if let Some(token_match) =
                            token_iter.peek_token_with_next_symbol(&next_symbol)?
                        {
                            let priority = token_iter
                                .get_terminal_def(&next_symbol)
                                .map(|terminal_def| terminal_def.priority)
                                .unwrap_or_default();
                            next_possible_symbols.push(SymbolTokenState {
                                symbol: next_symbol.clone(),
                                token_match,
                                state_index,
                                priority,
                            });
                        }
                        #[cfg(feature = "debug")]
                        if self.parser_config.debug {
                            token_arr.push(
                                next_possible_symbols
                                    .last()
                                    .map(|candidate| candidate.token_match.token.clone()),
                            );
                        }
                    }
                }
            }

            if next_possible_symbols.len() > 1 {
                next_possible_symbols.sort_by(|a, b| {
                    b.priority
                        .cmp(&a.priority)
                        .then_with(|| b.token_match.next_start.cmp(&a.token_match.next_start))
                });
            }

            if let Some(sym_tk_st) = next_possible_symbols.first() {
                let tk = sym_tk_st.token_match.token.clone();
                let state = chart[i].states[sym_tk_st.state_index].clone();

                self.scan(&mut chart, tk.clone(), &state, i);
                token_iter.commit_token_match(&sym_tk_st.token_match);

                j += 1;

                let priority = sym_tk_st.priority;
                let next_start = sym_tk_st.token_match.next_start;

                for alternative in next_possible_symbols.iter().skip(1) {
                    if priority != alternative.priority {
                        break;
                    }

                    if next_start != alternative.token_match.next_start {
                        break;
                    }

                    let tk = alternative.token_match.token.clone();
                    let state = chart[i].states[alternative.state_index].clone();
                    self.scan(&mut chart, tk.clone(), &state, i);
                }
            }

            #[cfg(feature = "debug")]
            if self.parser_config.debug {
                println!(
                    "Index: {} | {}",
                    i,
                    token_arr
                        .iter()
                        .filter(|x| x.is_some())
                        .map(|x| match x {
                            Some(x) => x.to_string(),
                            None => "None".to_string(),
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                for state in chart[i].states.iter() {
                    println!("\tState: {}", state);
                }
            }
            i += 1;
        }

        chart.pop();
        self.finalize_basic_parse(&chart, token_iter, &prev_next_symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::Algorithm;
    use crate::load_grammar::load_grammar;

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

    #[test]
    fn state_core_methods_and_display_work() {
        let rule = Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal("expr".to_string())),
            vec![
                Arc::new(Symbol::NonTerminal("expr".to_string())),
                Arc::new(Symbol::Terminal("INT".to_string())),
            ],
            Arc::new(RuleMeta::default()),
            0,
        ));
        let s0 = EarleyItem::new(rule.clone(), 0, 0, 0, vec![]);
        let s2 = EarleyItem::new(rule, 2, 0, 1, vec![]);

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
        let parser_opt = Arc::new(ParserConfig::default());
        let pf = test_frontend(grammar, parser_opt.clone());
        let earley = EarleyParser::new(pf.clone(), parser_opt);
        let mut tk = pf.tokenizer("x");
        assert!(earley.parse(&mut tk).is_ok());

        let explicit_opt = Arc::new(ParserConfig {
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Explicit,
            ..ParserConfig::default()
        });
        let explicit_pf = test_frontend(grammar, explicit_opt.clone());
        let explicit = EarleyParser::new(explicit_pf.clone(), explicit_opt);
        let mut tk = explicit_pf.tokenizer("x");
        let ast = explicit.parse(&mut tk).unwrap();
        assert_eq!(ast.tree_name(), Some(&"_ambiguity".to_string()));
    }

    #[test]
    fn earley_handles_contextual_terminals() {
        let grammar = r#"
        start: "select" NAME
        NAME: /[a-z]+/
        %import WS
        %ignore WS
        "#;

        let _opt = Arc::new(ParserConfig {
            ..ParserConfig::default()
        });
        let _pf = test_frontend(grammar, _opt.clone());
        let parser = EarleyParser::new(_pf.clone(), _opt);
        let mut tk = _pf.tokenizer("select users");
        assert!(parser.parse(&mut tk).is_ok());
    }

    #[test]
    fn earley_prefers_longer_same_priority_match_when_shorter_branch_cannot_finish() {
        let grammar = r#"
        start: AB C | A B
        AB: "ab"
        A: "a"
        B: "b"
        C: "c"
        "#;

        let parser_opt = Arc::new(ParserConfig::default());
        let pf = test_frontend(grammar, parser_opt.clone());
        let parser = EarleyParser::new(pf.clone(), parser_opt);
        let mut tk = pf.tokenizer("abc");

        assert!(parser.parse(&mut tk).is_ok());
    }

    #[test]
    fn finalize_basic_parse_returns_error_for_empty_chart() {
        let parser_opt = Arc::new(ParserConfig::default());
        let pf = test_frontend(
            r#"
            start: "x"
            "#,
            parser_opt.clone(),
        );
        let parser = EarleyParser::new(pf, parser_opt);
        let mut tk = parser.get_parser_frontend().tokenizer("x");

        let err = parser
            .finalize_basic_parse(&[], &mut tk, &[])
            .expect_err("empty chart should return an error");
        assert!(matches!(
            err,
            SwiftletError::Parse(ParseError::FailedToParse(_))
        ));
    }
}
