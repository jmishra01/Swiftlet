use crate::parser::error::ParserError;
use crate::{
    grammar::{Rule, RuleOption},
    lexer::{Symbol, Token, Tokenizer, AST},
    non_terms,
    parser::utils::dot_state,
    parser::Parser,
    parser_frontends::ParserFrontend,
    terms,
    ParserOption,
};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::sync::Arc;

// Type Alias
pub(crate) type SymbolSet = IndexSet<Arc<Symbol>>;
pub(crate) type SymbolMap = HashMap<Arc<Symbol>, Vec<(usize, Arc<Rule>)>>;
pub(crate) type ItemSet = HashSet<Arc<Item>>;
pub(crate) type VecItemSet = Vec<ItemSet>;
pub(crate) type Action = HashMap<(usize, Arc<Symbol>), IndexSet<ActionTable>>;
pub(crate) type GoTo = HashMap<(usize, Arc<Symbol>), usize>;
pub(crate) type First = HashMap<Arc<Symbol>, HashSet<Arc<Symbol>>>;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ActionTable {
    Shift(usize),
    Reduce(usize),
    Accepted,
}

impl ActionTable {
    /// Returns action kind label.
    fn name(&self) -> String {
        match self {
            ActionTable::Shift(_) => "Shift".to_string(),
            ActionTable::Reduce(_) => "Reduce".to_string(),
            ActionTable::Accepted => "Accepted".to_string(),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub(crate) struct Item {
    pub(crate) rule_id: usize,
    dot: usize,
    pub(crate) rule: Arc<Rule>,
    pub(crate) lookahead: Arc<Symbol>,
}

impl Item {
    /// Creates an LR item with lookahead.
    fn new(rule_id: usize, dot: usize, rule: Arc<Rule>, lookahead: Arc<Symbol>) -> Item {
        Item {
            rule_id,
            dot,
            rule,
            lookahead,
        }
    }

    /// Returns whether dot is at rule end.
    pub(crate) fn is_complete(&self) -> bool {
        self.dot == self.rule.len()
    }

    /// Returns whether `symbol` is the next expected symbol.
    pub(crate) fn is_next_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        if self.is_complete() {
            return false;
        }
        self.rule.expansion[self.dot] == *symbol
    }

    /// Returns next symbol after dot.
    pub(crate) fn next_symbol(&self) -> Option<&Arc<Symbol>> {
        if self.is_complete() {
            return None;
        }
        Some(&self.rule.expansion[self.dot])
    }

    /// Returns a new item with dot advanced by one.
    fn move_dot(&self) -> Option<Self> {
        if self.is_complete() {
            return None;
        }
        Some(Item::new(
            self.rule_id,
            self.dot + 1,
            self.rule.clone(),
            self.lookahead.clone(),
        ))
    }
}

impl Display for Item {
    /// Formats item as `rule_id; A -> alpha ● beta ; lookahead`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (rule, before_dot, after_dot) = dot_state(&self.rule, self.dot);
        write!(
            f,
            "{:>3}; {} -> {} ● {} ; {}",
            self.rule_id,
            rule,
            before_dot,
            after_dot,
            self.lookahead.as_ref().as_str()
        )
    }
}

/// Returns special end-of-input terminal symbol.
pub(crate) fn get_last_symbol() -> Arc<Symbol> {
    terms!("$END")
}

#[inline]
/// Collects grammar rules and constructs a symbol-to-rule index map.
pub(crate) fn setup(
    parser_frontend: Arc<ParserFrontend>,
    start: Arc<Symbol>,
) -> (Vec<Arc<Rule>>, SymbolMap) {
    let mut rules = parser_frontend.get_parser().get_all_expansion();

    let mut mapped: SymbolMap = HashMap::new();

    for (index, rule) in rules.iter().enumerate() {
        let r = mapped.entry(rule.origin.clone()).or_default();
        r.push((index, rule.clone()));
    }

    let augmented_grammar = Arc::new(Rule::new(
        non_terms!("gamma"),
        vec![start],
        Arc::new(RuleOption::default()),
        0,
    ));
    rules.push(augmented_grammar);
    (rules, mapped)
}

pub struct Clr {
    parser_frontend: Arc<ParserFrontend>,
    #[allow(dead_code)]
    parser_conf: Arc<ParserOption>,
    pub(crate) rules: Vec<Arc<Rule>>,
    pub(crate) mapped: SymbolMap,
    pub(crate) first: First,
    action: Action,
    goto: GoTo,
}

impl Clr {
    /// Creates a CLR parser and builds canonical items plus ACTION/GOTO tables.
    pub(crate) fn new(
        parser_frontend: Arc<ParserFrontend>,
        parser_conf: Arc<ParserOption>,
    ) -> Clr {
        let (rules, mapped) = setup(parser_frontend.clone(), non_terms!(parser_conf.start));
        let first = first_set(&rules);

        if parser_conf.debug {
            debug_clr_rules(&rules);
            debug_first_set(&first);
        }

        let mut clr = Clr {
            parser_frontend,
            parser_conf,
            rules,
            mapped,
            first,
            action: HashMap::new(),
            goto: HashMap::new(),
        };
        let (canonical_items, transitions) = canonical_items(&mut clr);

        if clr.parser_conf.debug {
            debug_canonical_and_transtion_sets(&canonical_items, &transitions);
        }

        let (action, goto) = clr.build_action_and_goto_table(&canonical_items, &transitions);
        clr.action = action;
        clr.goto = goto;

        clr
    }

    /// Returns ordered rule list used by parser tables.
    fn get_rules(&self) -> &Vec<Arc<Rule>> {
        &self.rules
    }

    #[inline]
    /// Returns FIRST set for a symbol.
    fn get_first(&self, seq: &Arc<Symbol>) -> Option<&HashSet<Arc<Symbol>>> {
        self.first.get(seq)
    }

    #[inline]
    /// Builds ACTION and GOTO tables from canonical items and transitions.
    fn build_action_and_goto_table(
        &self,
        canonical_items: &VecItemSet,
        transition: &GoTo,
    ) -> (Action, GoTo) {
        let mut action: Action = HashMap::new();
        let mut goto: GoTo = HashMap::new();

        for (index, item) in canonical_items.iter().enumerate() {
            for it in item.iter() {
                if it.is_complete() {
                    if it.rule.origin == non_terms!("gamma") {
                        let target = action.entry((index, get_last_symbol())).or_default();
                        target.insert(ActionTable::Accepted);
                    } else {
                        let target = action.entry((index, it.lookahead.clone())).or_default();
                        target.insert(ActionTable::Reduce(it.rule_id));
                    }
                } else if let Some(next_symbol) = it.next_symbol() &&
                    let Some(transition_index) = transition.get(&(index, next_symbol.clone())) {
                    if next_symbol.is_terminal() {
                        let target = action.entry((index, next_symbol.clone())).or_default();
                        target.insert(ActionTable::Shift(*transition_index));
                    } else {
                        goto.insert((index, next_symbol.clone()), *transition_index);
                    }
                }
            }
        }
        (action, goto)
    }

    /// Resolves a parser action from a possible conflict set using priorities.
    fn get_next_action<'a>(&self, lr_table: &'a IndexSet<ActionTable>) -> Result<&'a ActionTable, ParserError> {
        if lr_table.len() == 1 {
            return Ok(lr_table.first().unwrap());
        }
        let rules = lr_table
            .iter()
            .map(|x| {
                let n = match x {
                    ActionTable::Shift(n) => *n,
                    ActionTable::Reduce(n) => *n,
                    ActionTable::Accepted => 0usize,
                };
                (self.rules.get(n).unwrap(), x)
            })
            .collect::<Vec<(&Arc<Rule>, &ActionTable)>>();

        let vec_priority = rules
            .iter()
            .map(|(r, _)| r.rule_option.priority())
            .collect::<Vec<usize>>();

        let mut hs_priority = HashSet::new();

        for i in vec_priority.iter() {
            hs_priority.insert(*i);
        }

        if vec_priority.len() == hs_priority.len() {
            rules
                .iter()
                .max_by(|&x1, &x2| {
                    x1.0.rule_option
                        .priority()
                        .cmp(&x2.0.rule_option.priority())
                })
                .unwrap();
            Ok(rules.first().unwrap().1)
        } else {
            let conflict = lr_table
                .iter()
                .map(|x| x.name())
                .collect::<Vec<String>>()
                .join("-");
            Err(ParserError::Conflict { lr_table: lr_table.clone(), conflict })
        }
    }

    #[inline]
    /// Executes shift action and fetches next lookahead token.
    fn shift_action(&self,
                    pos: usize,
                    stack_states: &mut VecDeque<usize>,
                    stack_symbols: &mut VecDeque<AST>,
                    lookahead: &Arc<Token>,
                    tokenizer: &mut Tokenizer) -> Arc<Token> {
        stack_states.push_back(pos);
        stack_symbols.push_back(AST::Token(lookahead.clone()));
        if let Some(token) = tokenizer.next() {
            token
        } else {
            Arc::new(Token::new(
                get_last_symbol().as_ref().as_str().to_string(),
                0,
                0,
                0,
                get_last_symbol(),
            ))
        }
    }

    /// Executes reduce action and performs goto transition.
    fn reduce_action(&self,
                     pos: usize,
                     stack_states: &mut VecDeque<usize>,
                     stack_symbols: &mut VecDeque<AST>) -> Result<bool, ParserError> {
        let rule = self.rules.get(pos).unwrap();

        let mut children = Vec::new();
        for _ in 0..rule.expansion.len() {
            stack_states.pop_back();
            let ast = stack_symbols.pop_back().unwrap();
            if ast.is_start_with_underscore() {
                match ast {
                    AST::Tree(_, child) => {
                        for (index, c) in child.iter().enumerate() {
                            children.insert(index, c.clone());
                        }
                    }
                    _ => continue,
                }
            } else {
                children.insert(0, ast);
            }
        }

        if rule.rule_option.is_expand() && children.len() == 1 {
            stack_symbols.push_back(children[0].clone());
        } else {
            stack_symbols.push_back(AST::Tree(rule.origin.as_ref().as_str().to_string(), children));
        }

        if let Some(index) = stack_states.iter().last()
            && let Some(goto_state) = self.goto.get(&(*index, rule.origin.clone()))
        {
            stack_states.push_back(*goto_state);
        } else {
            return Err(ParserError::TransitionError(rule.origin.clone()));
        }
        Ok(true)
    }
}

impl Parser for Clr {
    /// Returns parser frontend.
    fn get_parser_frontend(&self) -> Arc<ParserFrontend> {
        self.parser_frontend.clone()
    }

    /// Runs CLR parse loop and returns AST or parser error.
    fn parse(&self, mut tokenizer: Tokenizer) -> Result<AST, ParserError> {
        let mut stack_states = VecDeque::from([0usize]);
        let mut stack_symbols: VecDeque<AST> = VecDeque::new();
        let mut lookahead = tokenizer.next().unwrap();

        loop {
            let state = *stack_states.iter().last().unwrap();
            if let Some(lr_table) = self.action.get(&(state, lookahead.terminal.clone())) {
                // Check Conflict SR & RR
                match self.get_next_action(lr_table) {
                    Ok(action) => {
                        match action {
                            ActionTable::Accepted => break,
                            ActionTable::Shift(pos) => {
                                lookahead = self.shift_action(*pos, &mut stack_states, &mut stack_symbols, &lookahead, &mut tokenizer);
                            }
                            ActionTable::Reduce(pos) => {
                                self.reduce_action(*pos, &mut stack_states, &mut stack_symbols)?;
                            }
                        }
                    }
                    Err(message) => {
                        return Err(message);
                    }
                }
            } else {
                return Err(ParserError::RuleNotFound(lookahead.word.clone()));
            }
        }
        if let Some(ast) = stack_symbols.pop_front() {
            return Ok(ast);
        }
        Err(ParserError::FailedToParse(tokenizer.get_text().to_string()))
    }
}

/// Computes closure for an item set and collects next transition symbols.
pub(crate) fn closure(
    lr_parser: &Clr,
    it_item: impl Iterator<Item=Arc<Item>>,
) -> (ItemSet, SymbolSet) {
    let mut next_symbols: SymbolSet = IndexSet::new();
    let mut items: ItemSet = HashSet::new();
    let mut cache: ItemSet = HashSet::new();

    for item in it_item {
        let mut added = true;
        items.insert(item);
        while added {
            added = false;
            let process_items = items.clone();
            for item in process_items.iter() {
                if !cache.insert(item.clone()) {
                    continue;
                }
                if item.is_complete() {
                    continue;
                }

                let next_symbol = item.next_symbol().unwrap();
                next_symbols.insert(next_symbol.clone());

                if !next_symbol.is_terminal()
                    && let Some(productions) = lr_parser.mapped.get(next_symbol)
                {
                    if let Some(v) = item.rule.expansion[item.dot + 1..].first() {
                        let lookahead = lr_parser.get_first(v).unwrap();
                        for (index, rule) in productions.iter() {
                            for lh in lookahead.iter() {
                                if items.insert(Arc::new(Item::new(
                                    *index,
                                    0,
                                    rule.clone(),
                                    lh.clone(),
                                ))) {
                                    added = true;
                                }
                            }
                        }
                    } else {
                        for (index, rule) in productions.iter() {
                            if items.insert(Arc::new(Item::new(
                                *index,
                                0,
                                rule.clone(),
                                item.lookahead.clone(),
                            ))) {
                                added = true;
                            }
                        }
                    }
                }
            }
        }
    }
    (items, next_symbols)
}

/// Expands canonical LR item sets recursively and records transitions.
fn find_canonical_items(
    lr_parser: &mut Clr,
    canonical_items: &mut VecItemSet,
    transitions: &mut GoTo,
    item_index: usize,
    item: &ItemSet,
    list_of_next_symbols: &SymbolSet,
) {
    for symbol in list_of_next_symbols.iter() {
        let (next_canonical_item, next_list_of_next_symbols) = closure(
            lr_parser,
            item.iter()
                .filter(|x1| x1.is_next_symbol(symbol))
                .map(|x2| Arc::new(x2.move_dot().unwrap())),
        );
        if next_canonical_item.is_empty() {
            continue;
        }
        if canonical_items.contains(&next_canonical_item) {
            let posi = canonical_items
                .iter()
                .position(|x| *x == next_canonical_item)
                .unwrap();
            transitions.insert((item_index, symbol.clone()), posi);
        } else {
            canonical_items.push(next_canonical_item.clone());
            let next_item_index = canonical_items.len() - 1;
            transitions.insert((item_index, symbol.clone()), next_item_index);
            find_canonical_items(
                lr_parser,
                canonical_items,
                transitions,
                next_item_index,
                &next_canonical_item,
                &next_list_of_next_symbols,
            );
        }
    }
}

/// Builds canonical collection of LR(1) item sets and transition graph.
pub(crate) fn canonical_items(lr_parser: &mut Clr) -> (VecItemSet, GoTo) {
    // Augmented grammar
    let first_items = [Arc::new(Item::new(
        lr_parser.get_rules().len() - 1,
        0,
        lr_parser.get_rules().iter().last().unwrap().clone(),
        get_last_symbol(),
    ))];

    let (first_items, list_of_next_symbols) = closure(lr_parser, first_items.iter().cloned());

    let mut canonical_items: VecItemSet = Vec::from([first_items.clone()]);
    let mut transitions: GoTo = HashMap::new();

    find_canonical_items(
        lr_parser,
        &mut canonical_items,
        &mut transitions,
        0,
        &first_items,
        &list_of_next_symbols,
    );

    (canonical_items, transitions)
}

/// Computes FIRST sets used during CLR closure expansion.
pub(crate) fn first_set(rules: &[Arc<Rule>]) -> First {
    let mut first: First = rules
        .iter()
        .map(|x| (x.origin.clone(), HashSet::new()))
        .collect();

    let mut added = true;

    while added {
        added = false;
        for rule in rules.iter() {
            let origin = &rule.origin;
            if let Some(e) = rule.expansion.first() {
                if e.is_terminal() {
                    if first.get_mut(origin).unwrap().insert(e.clone()) {
                        added = true;
                    }
                    let val = first.entry(e.clone()).or_default();
                    val.insert(e.clone());
                } else if !first[e].is_empty() {
                    let v_iter: HashSet<Arc<Symbol>> =
                        first.get(e).unwrap().iter().cloned().collect();

                    let val = first.get_mut(origin).unwrap();
                    for v in v_iter {
                        if val.insert(v) {
                            added = true;
                        }
                    }
                }
            }
            for t in rule.expansion[1..].iter().filter(|x| x.is_terminal()) {
                let val = first.entry(t.clone()).or_default();
                val.insert(t.clone());
            }
        }
    }
    first
}

// ---------------- CLR Debug ---------------- //
#[inline]
/// Prints numbered rules for debug tracing.
fn debug_clr_rules(rules: &[Arc<Rule>]) {
    println!("\nList of Rules in BNF format.");
    println!("============================");

    for (index, rule) in rules.iter().enumerate() {
        println!("\t{:<2}; {:?}", index, rule);
    }
    println!();
}

#[inline]
/// Prints FIRST sets for debug tracing.
fn debug_first_set(first: &First) {
    println!("First Set");
    println!("=========");
    for (k, v) in first.iter() {
        println!(
            "\t{:?} => {:?}",
            k.as_ref().as_str(),
            v.iter().map(|x| { x.as_ref().as_str().to_string() }).collect::<Vec<String>>()
        );
    }
    println!();
}

#[inline]
/// Prints canonical items and transitions for debug tracing.
fn debug_canonical_and_transtion_sets(canonical_items: &VecItemSet, transitions: &GoTo) {
    println!("Canonical Items:");
    println!("================");
    for (index, items) in canonical_items.iter().enumerate() {
        println!("I-{}:", index);
        for item in items.iter() {
            println!("\t{}", item);
        }
    }
    println!();

    println!("Transitions:");
    println!("============");
    for ((index, sym), transition) in transitions.iter() {
        println!("\t(I-{:<3}, {}): I-{}", index, sym.as_ref().as_str(), transition);
    }
}
// ----------------------------------------------- //
