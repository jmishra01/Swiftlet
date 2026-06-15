use crate::ast::Ast;
use crate::error::{LexerError, ParseError, SwiftletError};
use crate::grammar::{Rule, RuleMeta};
use crate::lexer::{Symbol, Token, TokenProbe, Tokenizer};
use crate::parser::ParserBackend;
use crate::parser::utils::dot_state;
use crate::parser_frontends::GrammarRuntime;
use crate::{Ambiguity, ParserConfig, non_terms};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::{Display, Formatter};
use std::iter::Iterator;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Wraps an `Arc<T>` and uses pointer identity for `Hash` and `Eq`.
///
/// Safe to use for `Rule` because all grammar `Arc<Rule>`s are interned
/// (the same logical rule always shares the same allocation).
#[derive(Debug, Clone)]
struct ArcPtr<T>(Arc<T>);

impl<T> Hash for ArcPtr<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}


impl<T> PartialEq for ArcPtr<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for ArcPtr<T> {}


/// Deduplication key for Earley items (ignores child trees / backpointers).
/// Uses pointer-based hashing for `rule` since all grammar `Arc<Rule>`s are interned.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct StateCore {
    rule: ArcPtr<Rule>,
    dot: usize,
    start: usize,
    end: usize,
}

/// Records how an Earley item are derived; used to reconstruct the AST after parsing.
///
/// This replaces the eager `children: Vec<Ast>` on each item. During parsing, we only
/// store lightweight pointers; the full parse tree is built in one pass at the end.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Backpointer {
    /// Seed or prediction item (dot = 0).
    Init,
    /// Produced by scanning a terminal; parent item is at `(parent_col, parent_item)`.
    Scan {
        parent_col: usize,
        parent_item: usize,
        token: Arc<Token>
    },
    /// Produced by completing a non-terminal.
    Complete {
        parent_col: usize,
        parent_item: usize,
        completer_col: usize,
        completer_item: usize,
    },
    /// Produced by a Leo transition: a deterministic right-recursive completion chain was
    /// collapsed into a single step. The skipped intermediate items are reconstructed during
    /// tree building by walking the Leo chain in `chain[leo_col].leo[leo_sym]`.
    Leo {
        /// Column whose Leo map holds the transition for `leo_sym` (= completer's origin).
        leo_col: usize,
        /// Transition symbol (= the completed non-terminal).
        leo_sym: Arc<Symbol>,
        /// The completed item that bottoms out the collapsed chain.
        completer_col: usize,
        completer_item: usize,
    }
}

/// A memoized Leo transition for one `(column, symbol)`.
///
/// `top_*` identifies the single item added to the chart when a completion uses this
/// transition (the topmost item of the collapsed right-recursive chain). `penult_*`
/// locates this level's penult item `[B -> β.A, k]`, which the reconstruction pass uses to
/// rebuild the skipped derivation. Every Leo item in one chain shares the same `top_*`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct LeoItem {
    top_rule: Arc<Rule>,
    top_origin: usize,
    penult_col: usize,
    penult_item: usize,
}


/// An Earley item with backpointers for deferred AST construction.
///
/// Children are **not** accumulated during parsing. Instead `backpointers` records
/// how this item was reached, and the full tree is reconstructed in a single
/// post-parse pass -- eliminating the O(n2) child-cloning cascade of the eager approach.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EarleyItem {
    pub rule: Arc<Rule>,
    pub dot: usize,
    pub start: usize,
    pub end: usize,
    /// Derivation records; always >= 1 entry after insertion.
    pub backpointers: Vec<Backpointer>
}

pub(crate) struct SymbolTokenState {
    symbol: Arc<Symbol>,
    probe: TokenProbe,
    state_index: usize,
    priority: usize,
}

/// Chart column for the deferred Earley parser.
#[derive(Clone, Default)]
struct ChartColumn {
    items: Vec<EarleyItem>,
    /// Maps `StateCore` -> index in ìtems`; provides O(1) deduplication.
    index: FxHashMap<StateCore, usize>,
    /// Maps each expected non-terminal -> indices of items waiting on it.
    pending_by_symbol: FxHashMap<Arc<Symbol>, Vec<usize>>,
    /// Non-terminals already predicted in this column; prevents redundant rule iteration.
    predicted: FxHashSet<Arc<Symbol>>,
    /// Memoized Leo transitions, keyed by the completed non-terminal. Populated lazily
    /// during completion; consulted again during tree reconstruction.
    leo: FxHashMap<Arc<Symbol>, LeoItem>
}

impl EarleyItem {
    /// Creates an Earley state.
    pub fn new(rule: Arc<Rule>, dot: usize, start: usize, end: usize) -> Self {
        Self {
            rule,
            dot,
            start,
            end,
            backpointers: Vec::new(),
        }
    }

    /// Returns whether the state has consumed the full rule expansion.
    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        self.dot == self.rule.len()
    }

    /// Returns the next expected symbol, if any.
    #[inline(always)]
    pub fn next_symbol(&self) -> Option<&Arc<Symbol>> {
        self.rule.expansion.get(self.dot)
    }
}

impl Display for EarleyItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (rule, before_dot, after_dot) = dot_state(&self.rule, self.dot);
        write!(f, "{rule} -> {before_dot} ● {after_dot}")
    }
}

impl ChartColumn {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            index: FxHashMap::default(),
            pending_by_symbol: FxHashMap::default(),
            predicted: FxHashSet::default(),
            leo: FxHashMap::default(),
        }
    }

    /// Inserts `item` into this oclumn, deduplicating by `Statecore`.
    ///
    /// - **Resolve mode**: first derivation wins; duplicate `Statecore`s are dropped.
    /// - **Explicit mode**: new backpointers are merged onto the existing item so all
    /// derivations remain available for tree reconstruction.
    ///
    /// Returns the items's index if a new slot was created, `None` if merged or dopped.
    fn insert(&mut self, mut item: EarleyItem, resolve: bool) -> Option<usize> {
        let core = StateCore {
            rule: ArcPtr(item.rule.clone()),
            dot: item.dot,
            start: item.start,
            end: item.end,
        };

        if let Some(&existing) = self.index.get(&core) {
            if !resolve {
                if let Some(bp) = item.backpointers.pop() {
                    self.items[existing].backpointers.push(bp);
                }
            }
            return None;
        }

        let idx = self.items.len();
        self.index.insert(core, idx);

        if let Some(next_sym) = item.rule.expansion.get(item.dot) {
            if !next_sym.is_terminal() {
                self.pending_by_symbol.entry(next_sym.clone()).or_default().push(idx);
            }
        }

        self.items.push(item);
        Some(idx)

    }
}

/// Earley parser implementation used for general context-free grammars.
pub struct EarleyParser {
    parser_frontend: Arc<GrammarRuntime>,
    parser_config: Arc<ParserConfig>,
    /// Cached augmented start rule (`gamma -> <start>`); avoids allocation on each `parse()` call.
    start_rule: Arc<Rule>,
}

impl EarleyParser {
    pub fn new(parser_frontend: Arc<GrammarRuntime>, parser_config: Arc<ParserConfig>) -> Self {
        let start_rule = Arc::new(Rule::new(
            Arc::new(Symbol::NonTerminal("gamma".to_string())),
            vec![non_terms!(parser_config.start)],
            Arc::new(RuleMeta::default()),
            0
        ));
        Self {
            parser_frontend,
            parser_config,
            start_rule
        }
    }

    /// Prediction: for each rule of `next_symbol`, add a dot-0 item into column `i`.
    ///
    /// Guarded by `ChartColumn::predicted` so each non-terminal is expanded at most once
    /// per column -- subsequent items expecting the same symbol skip rule iteration entirely.
    #[inline(always)]
    fn prediction(
        &self,
        chart: &mut Vec<ChartColumn>,
        next_symbol: &Arc<Symbol>,
        i: usize,
        resolve: bool,
    ) {
        // Item 1: skip if this non-terminal was already predicted in this column;
        if !chart[i].predicted.insert(next_symbol.clone()) {
            return;
        }

        // item 3: iterate the grammar rules directly -- no intermediate Vec allocation.
        // `self.parser_frontend` and `chart` are separate memory, so the immutable
        // iterator borrow and the mutable chart borrow can coexist.
        for rule in self.parser_frontend.get_parser().next_expansion(next_symbol).cloned() {
            chart[i].insert(
                EarleyItem {
                    rule,
                    dot: 0,
                    start: i,
                    end: i,
                    backpointers: vec![Backpointer::Init],
                },
                resolve
            );
        }
    }

    /// Completion: advance all items in `chart[state_start]` that were waiting on the
    /// non-terminal completed by `chart[i].items[item_idx]`.
    #[inline(always)]
    fn complete(
        &self,
        chart: &mut Vec<ChartColumn>,
        item_idx: usize,
        i: usize,
        resolve: bool,
    ) {
        let state_start = chart[i].items[item_idx].start;
        let rule_origin = chart[i].items[item_idx].rule.origin.clone();

        // --- Leo fast path -------------------------------------------------------
        // Only in resolve mode (Explicit keeeps every derivation via normal completion),
        // and only for a non-nullable comppleter (`state_start < i`), which guarantees the chain
        // columns are full build and avoids same-column (nullable) hazards.
        //
        // When a deterministic reigght-recursive chain exists, jump straight to its topmost
        // item in one step instead of walking 0(chain) intermediate completions.
        if resolve && state_start < i && self.ensure_leo(chart, state_start, &rule_origin) {
            let leo = &chart[state_start].leo[&rule_origin];
            let top_rule = leo.top_rule.clone();
            let top_origin = leo.top_origin;
            let dot = top_rule.len();
            chart[i].insert(
                EarleyItem {
                    rule: top_rule,
                    dot,
                    start: top_origin,
                    end: i,
                    backpointers: vec![Backpointer::Leo {
                    leo_col: state_start,
                        leo_sym: rule_origin,
                        completer_col: i,
                        completer_item: item_idx,
                    }],
                },
                resolve,
            );
            return;
        }

        // Collect parent data before mutating `chart[i]` to satisfy the borrow checker.
        let parent_data: Vec<(usize, Arc<Rule>, usize, usize)> = chart[state_start]
            .pending_by_symbol
            .get(&rule_origin)
            .map(|v| {
                v.iter()
                    .map(|&pi| {
                        let p = &chart[state_start].items[pi];
                        (pi, p.rule.clone(), p.dot, p.start)
                    })
                    .collect()
            })
            .unwrap_or_default();


        for (parent_idx, parent_rule, parent_dot, parent_start) in parent_data {
            chart[i].insert(
                EarleyItem {
                    rule: parent_rule,
                    dot: parent_dot + 1,
                    start: parent_start,
                    end: i,
                    backpointers: vec![Backpointer::Complete {
                        parent_col: state_start,
                        parent_item: parent_idx,
                        completer_col: i,
                        completer_item: item_idx,
                    }],
                },
                resolve,
            );
        }
    }

    /// Ensures a Leo transition for `(j, a)` is computed and memoized, returning whether one exists. A Leo transition
    /// exists only for a *deterministic right recursion*: at each level there must be exactly one
    /// item waiting on the transition symbol, and that item must be a penult (`A` is its final symbol,
    /// so advancing completes it).
    ///
    /// Implemented iteratively (no recursion) so arbitrarily deep right-recursive chains
    /// cannot over flow the stack. Every level walked shares the same topmost item, so the
    /// whole deterministic chain is memoized in one pass -- giving amortized O(n) total.
    fn ensure_leo(&self, chart: &mut Vec<ChartColumn>, j: usize, a: &Arc<Symbol>) -> bool {
        if chart[j].leo.contains_key(a) {
            return true;
        }

        // Walk up the deterministic penult chain, recording each level to memoize.
        let mut path: Vec<(usize, Arc<Symbol>, usize)> = Vec::new();
        let mut visited: FxHashSet<(usize, Arc<Symbol>)> = FxHashSet::default();
        let mut cur_col = j;
        let mut cur_sym = a.clone();

        // The topmost item, set either by inheriting an existing Leo item (Some) or, when the
        // chain ends at an undetermined level, by advancing the last collected penult (None).
        let inherited: Option<(Arc<Rule>, usize)> = loop {
            if let Some(leo) = chart[cur_col].leo.get(&cur_sym) {
                break Some((leo.top_rule.clone(), leo.top_origin));
            }

            if !visited.insert((cur_col, cur_sym.clone())) {
                // Unit-production cycle -- stop to avoid looping; chain ends here.
                break None;
            }
            // Unique item waiting on `cur_sym` in this column?
            let pi = match chart[cur_col].pending_by_symbol.get(&cur_sym) {
                Some(v) if v.len() == 1 => v[0],
                _ => break None,
            };
            let (dot, len, start, origin) = {
                let it = &chart[cur_col].items[pi];
                (it.dot, it.rule.len(), it.start, it.rule.origin.clone())
            };
            // Penult check: The transition symbol must be the rule's final symbol.
            if dot + 1 != len {
                break None;
            }
            path.push((cur_col, cur_sym.clone(), pi));
            cur_col = start;
            cur_sym = origin;
        };

        if path.is_empty() {
            return false;
        }

        // Resolve the shared topmost item.
        let (top_rule, top_origin) = match inherited {
            Some(top) => top,
            None => {
                let (lc, _, lpi) = path.last().unwrap();
                let it = &chart[*lc].items[*lpi];
                (it.rule.clone(), it.start)
            }
        };

        for (col, sym, pi) in path {
            chart[col].leo.insert(
                sym,
                LeoItem {
                    top_rule: top_rule.clone(),
                    top_origin,
                    penult_col: col,
                    penult_item: pi,
                },
            );
        }
        true
    }



    /// Scan: create the advanced item in column `col + 1` by consuming `token`.
    #[inline(always)]
    fn scan(&self, chart: &mut Vec<ChartColumn>, token: Arc<Token>,
            parent_idx: usize, col: usize, resolve: bool) {
        // Clone fields before the mutable borrow of chart.
        let (parent_rule, parent_dot, parent_start) = {
            let p = &chart[col].items[parent_idx];
            (p.rule.clone(), p.dot, p.start)
        };

        let advanced = EarleyItem {
          rule: parent_rule,
            dot: parent_dot + 1,
            start: parent_start,
            end: col + 1,
            backpointers: vec![
                Backpointer::Scan {
                    parent_col: col,
                    parent_item: parent_idx,
                    token
                }
            ],
        };
        if chart.get(col + 1).is_none() {
            chart.push(ChartColumn::new());
        }
        chart[col + 1].insert(advanced, resolve);
    }

    // -- Tree reconstruction --------------------------------------------------------

    /// Returns the sequence of AST nodes for `rule.expansion[0..dot]` of an item.
    /// following the **first** backpointer (used in resolve / single-derivation mode).
    ///
    /// Recursion depth equals `item.dot` (<= rule length); no stack overflow risk.
    fn build_sequence(&self, chart: &[ChartColumn], col: usize, item_idx: usize) -> Vec<Ast> {
        match chart[col].items[item_idx].backpointers.first() {
            None | Some(Backpointer::Init) => vec![],
            Some(Backpointer::Scan { parent_col, parent_item, token }) => {
                let (pc, pi, tok) = (*parent_col, *parent_item, token.clone());
                let mut seq = self.build_sequence(chart, pc, pi);
                // Item 4: precomputed flag avoids two starts_with calls per token.
                if !tok.terminal_is_hidden {
                    seq.push(Ast::Token(tok));
                }
                seq
            }
            Some(Backpointer::Complete {parent_col, parent_item, completer_col, completer_item}) => {
                let (pc, pi, cc, ci) = (*parent_col, *parent_item, *completer_col, *completer_item);
                let parent_expand = chart[pc].items[pi].rule.expand;
                let parent_origin = chart[pc].items[pi].rule.origin.clone();
                let mut seq = self.build_sequence(chart, pc, pi);
                let contrib = self.contribution(chart, cc, ci, parent_expand, &parent_origin);
                seq.extend(contrib);
                seq
            }
            Some(Backpointer::Leo {
                leo_col,
                leo_sym,
                completer_col,
                completer_item
                 }) => {
                let (lc, ls, cc, ci) = (*leo_col, leo_sym.clone(), *completer_col, *completer_item);
                self.leo_sequence(chart, lc, &ls, cc, ci)
            }
        }
    }

    /// Reconstructs the child sequence of a Leo-collapsed topmost item by walking the Leo
    /// chain and rebuilding each skipped intermediate completion bottom-up.
    fn leo_sequence(&self, chart: &[ChartColumn], leo_col: usize, leo_sym: &Arc<Symbol>,
    completer_col: usize, completer_item: usize
    ) -> Vec<Ast> {
        // Collect the penult chain bottom->top, following the same links as `ensure_leo`.
        let mut chain: Vec<(usize, usize)> = Vec::new();
        let mut cur_col = leo_col;
        let mut cur_sym = leo_sym.clone();

        loop {
            let leo = &chart[cur_col].leo[&cur_sym];
            let (pcol, pitem) = (leo.penult_col, leo.penult_item);
            chain.push((pcol, pitem));
            let penult = &chart[pcol].items[pitem];
            let (k, b) = (penult.start, penult.rule.origin.clone());
            if chart.get(k).is_some_and(|c| c.leo.contains_key(&b)) {
                cur_col = k;
                cur_sym = b;
            } else {
                break;
            }
        }

        // Bottom level: the penult's prefix plus the bottoming-out completer's contribution.
        let (pb_col, pb_item) = chain[0];
        let pb_rule = chart[pb_col].items[pb_item].rule.clone();
        let mut current_seq = self.build_sequence(chart, pb_col, pb_item);
        let comp = self.contribution(chart, completer_col, completer_item, pb_rule.expand, &pb_rule.origin);
        current_seq.extend(comp);
        let mut below_rule = pb_rule;

        // Fold upward: each level wraps the level below as its trailing child.
        for &(p_col, p_item) in &chain[1..] {
            let p_rule = chart[p_col].items[p_item].rule.clone();
            let contrib = self.wrap_contribution(&below_rule, current_seq, p_rule.expand, &p_rule.origin);
            let mut seq = self.build_sequence(chart, p_col, p_item);
            seq.extend(contrib);
            current_seq = seq;
            below_rule = p_rule;
        }
        current_seq
    }

    /// Computes the AST node(s) a completed chart item contributes to its parent.
    fn contribution(
        &self,
        chart: &[ChartColumn],
        col: usize,
        item_idx: usize,
        parent_expand: bool,
        parent_origin: &Arc<Symbol>
    ) -> Vec<Ast> {
        let seq = self.build_sequence(chart, col, item_idx);
        let rule = chart[col].items[item_idx].rule.clone();
        self.wrap_contribution(&rule, seq, parent_expand, parent_origin)
    }

    /// Applies the hidden / expand / alias tree-transformation rules to a completed rule's
    /// child sequence, yielding the node(s) it contributes to its parent. Shared by chart-item
    /// completion and Leo-chain recontribution (which synthesizes items not present in the chart).
    fn wrap_contribution(
        &self,
        rule: &Rule,
        seq: Vec<Ast>,
        parent_expand: bool,
        parent_origin: &Arc<Symbol>
    ) -> Vec<Ast> {
        if rule.is_hidden || (parent_expand && &rule.origin == parent_origin) {
            seq
        } else if seq.len() == 1 && rule.expand {
            seq
        } else if seq.len() == 1
            && let Some(Ast::Tree(name, _)) = seq.first()
            && let Some(alias) = rule.rule_option.alias_rule()
            && alias.contains(name) {
            seq
        } else {
            vec![Ast::Tree(rule.origin.as_str().to_string(), seq)]
        }
    }

    /// Like `build_sequence` but returns **every** derivation (for `Ambiguity::Explicit`)
    fn build_all_sequence(
        &self,
        chart: &[ChartColumn],
        col: usize,
        item_idx: usize,
    ) -> Vec<Vec<Ast>> {
        // Cone backpointers so we can call self methods while chart is also borrowed.
        let backpointers = chart[col].items[item_idx].backpointers.clone();
        let mut results: Vec<Vec<Ast>> = Vec::new();

        for bp in &backpointers {
            match bp {
                Backpointer::Init => results.push(vec![]),
                Backpointer::Scan {parent_col, parent_item, token} => {
                    let (pc, pi, tok) = (*parent_col, *parent_item, token.clone());
                    for mut seq in self.build_all_sequence(chart, pc, pi) {
                        // item 4: precomputed flag avoids two starts_with calls per token.
                        if !tok.terminal_is_hidden {
                            seq.push(Ast::Token(tok.clone()));
                        }
                        results.push(seq);
                    }
                }
                Backpointer::Complete {
                    parent_col,
                    parent_item,
                    completer_col,
                    completer_item
                } => {
                    let (pc, pi, cc, ci) = (*parent_col, *parent_item, *completer_col, *completer_item);
                    let parent_expand = chart[pc].items[pi].rule.expand;
                    let parent_origin = chart[pc].items[pi].rule.origin.clone();
                    let parent_seqs = self.build_all_sequence(chart, pc, pi);
                    let contribs = self.contribution_all(chart, cc, ci, parent_expand, &parent_origin);
                    for parent_seq in &parent_seqs {
                        for contrib in &contribs {
                            let mut seq = parent_seq.clone();
                            seq.extend(contrib.clone());
                            results.push(seq);
                        }
                    }
                }

                // Leo transitions are only created in resolve mode, so this is unreachable in
                // Explicit mode; reconstruct the single deterministic derivation defensively.
                Backpointer::Leo {
                    leo_col,
                    leo_sym,
                    completer_col,
                    completer_item,
                } => {
                    let seq = self.leo_sequence(chart, *leo_col, leo_sym, *completer_col, *completer_item);
                    results.push(seq);
                }
            }
        }
        results
    }

    /// Like `contribution` but returns all contributions (for `Ambiguity::Explicit`).
    fn contribution_all(
        &self,
        chart: &[ChartColumn],
        col: usize,
        item_idx: usize,
        parent_expand: bool,
        parent_origin: &Arc<Symbol>
    ) -> Vec<Vec<Ast>> {
        let is_hidden = chart[col].items[item_idx].rule.is_hidden;
        let origin_eq = &chart[col].items[item_idx].rule.origin == parent_origin;
        let expand = chart[col].items[item_idx].rule.expand;
        let origin_str = chart[col].items[item_idx].rule.origin.as_str().to_string();
        let alias = chart[col].items[item_idx].rule.rule_option.alias_rule().map(|a| a.to_vec());

        self.build_all_sequence(chart, col, item_idx)
            .into_iter()
            .map(|seq| {
                if is_hidden || (parent_expand && origin_eq) {
                    seq
                } else if seq.len() == 1 && expand {
                    seq
                } else if seq.len() == 1
                    && let Some(Ast::Tree(name, _)) = seq.first()
                    && let Some(ref aliases) = alias
                    && aliases.contains(name) {
                    seq
                } else {
                    vec![Ast::Tree(origin_str.clone(), seq)]
                }
            }).collect()
    }

    fn finalize_basic_parse(
        &self,
        chart: &[ChartColumn],
        tokenizer: &mut Tokenizer,
        expected_token: &[Arc<Symbol>],
    ) -> Result<Ast, SwiftletError> {
        let Some(last_col) = chart.last() else {
            return Err(ParseError::FailedToParse(
                "earley parser produced no chart columns".to_string(),
            )
            .into());
        };

        match self.parser_config.ambiguity {
            Ambiguity::Resolve => {
                if let Some((acc_idx, _)) = last_col
                    .items
                    .iter()
                    .enumerate()
                    .find(|(_, it)| it.rule.origin.as_str() == "gamma" && it.is_complete()) {
                    let seq = self.build_sequence(chart, chart.len() - 1, acc_idx);
                    if let Some(seq) = seq.into_iter().next() {
                        return Ok(seq);
                    }
                }
            }
            Ambiguity::Explicit => {
                let last_col_idx = chart.len() - 1;
                let acc_indices: Vec<usize> = last_col
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, it)| it.rule.origin.as_str() == "gamma" && it.is_complete())
                    .map(|(i, _)| i)
                    .collect();

                if !acc_indices.is_empty() {
                    let mut all_tress: Vec<Ast> = Vec::new();
                    for acc_idx in acc_indices {
                        let seqs = self.build_all_sequence(chart, last_col_idx, acc_idx);
                        all_tress.extend(seqs.into_iter().flatten());
                    }
                    if !all_tress.is_empty() {
                        return if all_tress.len() == 1 {
                            Ok(all_tress.remove(0))
                        } else { Ok(Ast::Tree("_ambiguity".to_string(), all_tress))}
                    }
                }
            }
        }

        let exp = expected_token
            .iter()
            .map(|x| tokenizer.get_terminal_def(x).unwrap().value.clone())
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
    fn get_parser_frontend(&self) -> &Arc<GrammarRuntime> {
        &self.parser_frontend
    }

    /// Runs the Earley algorithm with deferred tree construction.
    ///
    /// Items are processed sequentially in each column (FIFO); newly added items
    /// are picked up automatically without an explicit worklist allocation.
    fn parse(&self, token_iter: &mut Tokenizer) -> Result<Ast, SwiftletError> {
        let mut chart = vec![ChartColumn::new()];
        let resolve = matches!(self.parser_config.ambiguity, Ambiguity::Resolve);

        chart[0].insert(
            EarleyItem {
                rule: self.start_rule.clone(),
                dot: 0,
                start: 0,
                end: 0,
                backpointers: vec![Backpointer::Init],
            },
            resolve,
        );

        let mut j = 1_usize;
        let mut i = 0_usize;
        let mut next_possible_symbols: Vec<SymbolTokenState> = Vec::new();
        let mut prev_next_symbol: Vec<Arc<Symbol>> = Vec::new();

        #[cfg(feature = "debug")]
        if self.parser_config.debug {
            println!("\nEarley Parser (deferred tree building)");
            println!("======================================");
        }

        while i <= j {
            if chart.get(i).is_none() {
                chart.push(ChartColumn::new());
            }

            if !next_possible_symbols.is_empty() {
                prev_next_symbol.clear();
                prev_next_symbol.extend(
                    next_possible_symbols
                        .iter()
                        .map(|c| c.symbol.clone()));
            }
            next_possible_symbols.clear();

            // Process all items in column i, including those added during this loop.
            let mut wi = 0;
            while wi < chart[i].items.len() {
                let is_complete = chart[i].items[wi].is_complete();
                let next_sym = chart[i].items[wi].next_symbol().cloned();

                if is_complete {
                    self.complete(&mut chart, wi, i, resolve);
                } else if let Some(next_sym) = next_sym {
                    // Item 2: O(1) enum check replaces a HashMap lookup.
                    if !next_sym.is_terminal() {
                        self.prediction(&mut chart, &next_sym, i, resolve);
                    } else if let Some(probe) = token_iter.peek_probe(&next_sym) {
                        // Allocation-free probe; the Token is built only if this candidate wins.
                        let priority = probe.priority;
                        next_possible_symbols.push(SymbolTokenState {
                            symbol: next_sym,
                            probe,
                            state_index: wi,
                            priority,
                        });
                    }
                }
                wi += 1;
            }


            if next_possible_symbols.len() > 1 {
                next_possible_symbols.sort_by(|a, b| {
                    b.priority
                        .cmp(&a.priority)
                        .then_with(|| b.probe.next_start.cmp(&a.probe.next_start))
                });
            }

            if !next_possible_symbols.is_empty() {
                // Build the Token only for the winning candidate(s); the rest never allocate.
                let best = &next_possible_symbols[0];
                let best_probe = best.probe;
                let best_priority = best.priority;

                let tk = token_iter.build_token(
                    best_probe.start,
                    best_probe.next_start,
                    best_probe.line,
                    &best.symbol
                );
                self.scan(&mut chart, tk, best.state_index, i, resolve);
                token_iter.commit_position(best_probe.next_start, best_probe.next_line);
                j += 1;

                for k in 1..next_possible_symbols.len() {
                    let alt = &next_possible_symbols[k];
                    if alt.priority != best_priority  || alt.probe.next_start != best_probe.next_start {
                        break;
                    }
                    let tk = token_iter.build_token(
                        alt.probe.start,
                        alt.probe.next_start,
                        alt.probe.line,
                        &alt.symbol
                    );
                    self.scan(&mut chart, tk, alt.state_index, i, resolve);
                }
            }

            #[cfg(feature = "debug")]
            if self.parser_config.debug {
                println!("Index: {}", i);
                for item in &chart[i].items {
                    println!("\tState: {}", item);
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
        let s0 = EarleyItem::new(rule.clone(), 0, 0, 0);
        let s2 = EarleyItem::new(rule, 2, 0, 1);

        assert!(!s0.is_complete());
        assert_eq!(s0.next_symbol().unwrap().as_str(), "expr");
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
        assert_eq!(ast.tree_name(), Some("_ambiguity"));
    }

    #[test]
    fn earley_handles_contextual_terminals() {
        let grammar = r#"
        start: "select" NAME
        NAME: /[a-z]+/
        %import WS
        %ignore WS
        "#;

        let _opt = Arc::new(ParserConfig { ..ParserConfig::default() });
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
