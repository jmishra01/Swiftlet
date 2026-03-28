use crate::grammar::{Rule, RuleOption};
use crate::lexer::{RegexFlag, Symbol, TerminalDef, get_symbol};
use crate::ast::AST;
use crate::{terminal_def};
use fancy_regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use crate::error::ParserError;

static ESCAPE: LazyLock<Regex> = LazyLock::new(|| {
    let re = Regex::new(r"(\p{P})").unwrap();
    re
});

pub type OptVecStr = Option<Vec<String>>;
pub type OptStr = Option<String>;


pub(crate) fn fetch_terminals(ast: &AST) -> Vec<String> {
    match ast {
        AST::Tree(_, childs) => {
            childs
                .iter()
                .map(fetch_terminals)
                .flatten()
                .collect::<Vec<_>>()
        },
        AST::Token(token) => {
            let word = token.word();
            vec![if word.starts_with("\"") & word.ends_with("\"") {
                    word[1..word.len() - 1].to_string()
                } else {
                    word.to_string()
                }]
        },
    }
}

pub struct TerminalCompiler<'a> {
    terminals: Vec<&'a AST>,
    map: HashMap<String, Arc<TerminalDef>>,
    index: usize,
    terminal_len: usize
}

impl<'a> TerminalCompiler<'a> {
    pub fn new(terminals: Vec<&'a AST>) -> Self {
        let terminal_len = terminals.len();
        Self { terminals, map: HashMap::new(), index: 0,  terminal_len}
    }

    pub fn get_terminals(&self) -> Vec<Arc<TerminalDef>> {
        self
            .map
            .values()
            .map(|t| t.clone())
            .collect::<Vec<_>>()
    }

    pub fn compile(&mut self) {
        while self.index < self.terminal_len {
            let terminal = self.terminals[self.index];
            self._transform(terminal);
            self.index += 1;
        }
    }

    fn _string(&mut self, child: &[AST]) -> OptStr {
        let first = child.first().unwrap();
        if let Some(w) = self._transform(first) {
            if w.ends_with("\"i") {
                let word = &w.as_str()[1..w.len()-2];
                let word  = ESCAPE.replace_all(word, r"\$1");
                return Some(format!("(?i:{})", word));
            }
            let word = &w.as_str()[1..w.len()-1];
            let word  = ESCAPE.replace_all(word, r"\$1");
            return Some(word.to_string());
        }
        None
    }

    fn _regex(&mut self, child: &[AST]) -> OptStr {
        let first = child.first().unwrap();
        let pattern = self._transform(first)?;
        let mut pattern = pattern.as_str();
        pattern = pattern.strip_prefix("/")?;

        let regex_flag_match = Regex::new(r"/[imsux]*$").unwrap();
        let captures = regex_flag_match.captures(pattern).unwrap().unwrap();
        let capture = captures.get(0).unwrap().as_str();
        pattern = pattern.strip_suffix(capture)?;
        let mut flags = String::new();

        for flag in "isux".chars() {
            if capture.contains(flag) {
                flags = format!("{}{}", flags, flag);
            }
        }

        let mut pattern = if flags.len() > 0 {
            format!("(?{}:{})", flags, pattern.to_string())
        } else {
            pattern.to_string()
        };
        if capture.contains("m") {
            pattern = format!("(?m:^{})", pattern);
        }
        Some(pattern)
    }


    fn _maybe(&mut self, child: &[AST]) -> OptStr {
        let first = child.first().unwrap();
        let maybe = self._transform(first)?;
        Some(format!("[{}]", maybe))
    }

    fn _op_expansion(&mut self, child: &[AST]) -> OptStr {
        let first = child.first().unwrap();
        let second = child.last().unwrap();
        let sign = self._transform(second)?;

        match first {
            AST::Tree(name, _) => {
                match name.as_str() {
                    "terminal" => {
                        let terminal = self._transform(first)?;
                        let is_contains = self.map.contains_key(&terminal);
                        if !is_contains {
                            for index in (self.index + 1)..self.terminal_len {
                                let terminal = self.terminals[index];
                                self._transform(terminal);
                            }
                        }
                        let value = self.map.get(&terminal).unwrap();
                        Some(format!("({}){}", value.value, sign))
                    },
                    _ => {
                        let value = self._transform(first)?;
                        Some(format!("{}{}", value, sign))
                    },
                }
            }
            _ => unreachable!()
        }
    }

    fn _or_expansion(&mut self, child: &[AST]) -> OptStr {
        let val = self._resolve_internal_terminal(child)?;
        if val.len() > 1 {
            return Some(format!("({})", val.join("|")));
        }
        Some(val.first().unwrap().to_string())
    }

    fn _expansion(&mut self, child: &[AST]) -> OptStr {
        match self._resolve_internal_terminal(child) {
            Some(arr) => Some(arr.join("")),
            None => None
        }
    }

    fn _resolve_internal_terminal(&mut self, child: &[AST]) -> OptVecStr {

        Some(
            child
            .iter()
            .filter_map(|c| {
                match c {
                    AST::Token(_) => None,
                    AST::Tree(name, childs) => {
                        match name.as_str() {
                            "terminal" => {
                                let child_terminal_name = self._terminal(childs).unwrap();
                                if !self.map.contains_key(&child_terminal_name) {
                                    for _index in (self.index + 1)..self.terminal_len {
                                        let term = self.terminals[_index];
                                        match term {
                                            AST::Tree(_, term_children) => {
                                                let &_first = term.get_child_tree("terminal")?.first()?;
                                                let token_word = self._transform(_first)?;
                                                if token_word.cmp(&child_terminal_name).is_eq() {
                                                    self._term(term_children);
                                                }
                                            },
                                            _ => unreachable!()
                                        }
                                    }
                                }
                                match self.map.get(&child_terminal_name) {
                                    Some(term) => Some(term.value.clone()),
                                    None => None
                                }
                            },
                            _ => self._transform(c)
                        }
                    }
                }
            })
            .collect::<Vec<_>>()
        )
    }

    fn _term(&mut self, child: &[AST]) -> OptStr {
        let first_child = child.first().unwrap();
        let name_term = self._transform(first_child)?;
        if self.map.contains_key(&name_term) {
            return None
        }

        let second_child = child.last().unwrap();
        let value = self._transform(second_child)?;
        let terminal_def = TerminalDef::with_regex(&name_term,
                                                   &value, RegexFlag::default(), 5);
        self.map.insert(name_term, Arc::new(terminal_def));
        None
    }

    fn _terminal(&mut self, child: &[AST]) -> OptStr {
        self._transform(child.first().unwrap())
    }

    fn _range(&mut self, child: &[AST]) -> OptStr {
        let first = child.first().unwrap();
        let first_range = self._transform(first)?;
        let first_range = first_range[1..first_range.len()-1].to_owned();

        let second = child.last().unwrap();
        let second_range = self._transform(second)?;
        let second_range = second_range[1..second_range.len()-1].to_owned();

        Some(format!("[{}-{}]", first_range, second_range))
    }

    fn _transform(&mut self, ast: &AST) -> OptStr {
        match ast {
            AST::Token(token) => Some(token.word().to_string()),
            AST::Tree(name, children) => {
                match name.as_str() {
                    "term" => self._term(children),
                    "string" => self._string(children),
                    "expansion" => self._expansion(children),
                    "or_expansion" => self._or_expansion(children),
                    "op_expansion" => self._op_expansion(children),
                    "terminal" => self._terminal(children),
                    "regex" => self._regex(children),
                    "maybe" => self._maybe(children),
                    "range" => self._range(children),
                    _ => panic!("{} tree not found.", name)
                }
            }
        }
    }
}

/// Normalizes origin symbol and derives rule options from name and priority.
fn origin_apply(name: &str, priority: usize, alias_rule: OptVecStr) -> (String, Arc<RuleOption>) {
    let is_expand = name.starts_with("?");
    let rule_option = Arc::new(RuleOption::new(is_expand, priority, alias_rule));
    let clean_name = if is_expand {
        name.strip_prefix("?").unwrap().to_string()
    } else {
        name.to_string()
    };

    (clean_name, rule_option)
}

/// Converts grammar-parser AST nodes into parser rules and terminal definitions.
pub struct RuleCompiler {
    terminal: Vec<Arc<TerminalDef>>,
    count: usize,
    cache: HashMap<String, String>,
    rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>,
}

impl RuleCompiler {
    /// Creates an empty transformer with shared built-in terminals.
    pub fn new() -> Self {
        Self {
            terminal: Vec::new(),
            count: 0usize,
            cache: HashMap::new(),
            rules: HashMap::new(),
        }
    }

    /// Returns transformed grammar and validates non-terminal references.
    pub fn get_grammar(&self) -> Result<HashMap<Arc<Symbol>, Vec<Arc<Rule>>>, ParserError> {
        for v in self.rules.values() {
            for r in v.iter() {
                for e in r.expansion.iter() {
                    if !e.is_terminal() && !self.rules.contains_key(e) {
                        return Err(ParserError::RuleProductionNotFound(e.as_str().to_string()))
                    }
                }
            }
        }

        Ok(self.rules.clone())
    }

    /// Returns generated terminals.
    pub fn get_terminal(&self) -> Vec<Arc<TerminalDef>> {
        self.terminal.clone()
    }

    #[inline]
    /// Transforms a terminal or non-terminal leaf node.
    fn terminal_non_terminal(&mut self, tree: &[AST]) -> OptVecStr {
        self._transform(&tree[0])
    }

    #[inline]
    /// Expands concatenated expressions into full production combinations.
    fn expansions(&mut self, tree: &[AST]) -> OptVecStr {
        if tree.len() == 1 {
            let ret = self._transform(&tree[0]);
            return ret;
        }
        let mut v_result = Vec::from(["".to_string()]);
        for node in tree.iter() {
            if let Some(t) = self._transform(node) {
                v_result = t
                    .iter()
                    .flat_map(|x| {
                        v_result
                            .iter()
                            .map(|y| format!("{} {}", y, x.clone()).trim().to_string())
                    })
                    .collect::<Vec<String>>();
            }
        }

        Some(v_result)
    }

    #[inline]
    /// Transforms OR alternatives and memoizes generated helper rules.
    fn or_expansion(&mut self, tree: &[AST]) -> OptVecStr {
        if tree.len() == 1 {
            return self._transform(&tree.first().unwrap());
        }

        let result: Vec<String> = tree
            .iter()
            .filter_map(|x| self._transform(x))
            .flatten()
            .collect();
        let or_text = result.join("|");

        if let Some(rule_name) = self.cache.get(&or_text) {
            return Some(vec![rule_name.clone()]);
        }

        self.count += 1;
        let rule_name = format!("_or_expr_{}", self.count);
        self.cache.insert(or_text, rule_name.clone());
        self.insert_rules(rule_name.as_str(), result, 0, None);

        Some(vec![rule_name])
    }

    #[inline]
    /// Expands postfix operators (`?`, `+`, `*`) into helper productions.
    fn op_expansion(&mut self, tree: &[AST]) -> OptVecStr {
        let second = self._transform(&tree[1]).unwrap()[0].clone();

        if let AST::Tree(name, _) = &tree[0]
            && name.cmp(&"terminal".to_string()).is_eq()
        {
            let v_result = self._transform(&tree[0]).unwrap()[0].clone();

            if let Some(r) = self
                .terminal
                .iter()
                .find(|&x| x.get_name().as_ref().as_str() == v_result.as_str())
            {
                return Some(vec![format!("{}{}", r.value, second).trim().to_string()]);
            }
        }

        let mut v_result = Vec::from([self._transform(&tree[0]).unwrap().join(" ")]);

        let second = second.trim();

        match second {
            "?" => {
                v_result.push("".to_string());
                Some(v_result)
            }
            "+" | "*" => {
                let e = &v_result[0];
                self.count += 1;
                let r = format!("_expr_{}_{}", second.trim(), self.count);

                self.insert_rules(
                    r.as_str(),
                    vec![format!("{} {}", r, e), e.to_string()],
                    0,
                    None,
                );

                match second {
                    "+" => Some(vec![r]),
                    _ => Some(vec![r, "".to_string()]),
                }
            }
            _ => None,
        }
    }

    /// Inserts a new production list into the rule map.
    fn insert_rules(
        &mut self,
        rule_name: &str,
        prod: Vec<String>,
        priority: usize,
        alias_rules: OptVecStr,
    ) {
        let (clean_name, rule_option) = origin_apply(rule_name, priority, alias_rules);

        let expansion = prod
            .iter()
            .enumerate()
            .map(|(order, exp)| {
                Arc::new(Rule::new(
                    Arc::new(Symbol::NonTerminal(clean_name.clone())),
                    exp.split(" ").map(get_symbol).collect(),
                    rule_option.clone(),
                    order,
                ))
            })
            .collect::<Vec<Arc<Rule>>>();

        self.rules
            .insert(get_symbol(clean_name.as_str()), expansion);
    }

    #[inline]
    /// Transforms a rule declaration node.
    fn rule(&mut self, tree: &[AST]) -> OptVecStr {
        let rule_name = self._transform(&tree[0]).unwrap();
        let child = tree.last().unwrap();

        let mut alias_rules = vec![];

        if let Some(alias_trees) = child.get_child_tree("alias") {
            for alias_tree in alias_trees {
                if let Some(AST::Tree(_, alias_childs)) = alias_tree.get_last_child() &&
                    let Some(AST::Token(alias_child)) = alias_childs.last() {
                    alias_rules.push(alias_child.word().to_string());
                }
            }
        }


        let prod = self._transform(child).unwrap();
        let priority = if tree.len() > 2 {
            let priority = self._transform(&tree[1]).unwrap();
            priority[0].as_str().parse::<usize>().unwrap()
        } else {
            0
        };

        self.insert_rules(rule_name[0].as_str(), prod, priority, Some(alias_rules));

        None
    }

    #[inline]
    /// Transforms a string range node into a character class regex.
    fn range(&mut self, tree: &[AST]) -> OptVecStr {
        let first = self._transform(&tree[0]).unwrap();
        let second = self._transform(&tree[1]).unwrap();

        let clean = |x: &String| {
            x.strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_string()
        };

        let first = clean(first.first().unwrap());
        let second = clean(second.first().unwrap());

        let range = format!("[{first}-{second}]");

        Some(vec![range])
    }

    #[inline]
    /// Transforms a string literal into a synthetic terminal.
    fn string(&mut self, tree: &[AST]) -> OptVecStr {
        let word = self._transform(&tree[0]).unwrap();
        let word = word[0].strip_prefix("\"")?;
        let is_case_insensitive = word.ends_with("\"i");
        let word = if is_case_insensitive {
            word.strip_suffix("\"i")?
        } else {
            word.strip_suffix("\"")?
        };
        let terminal_name = word.to_uppercase();

        if is_case_insensitive {
            self.terminal.push(terminal_def!(
                terminal_name.as_str(),
                word,
                RegexFlag {
                    i: is_case_insensitive,
                    ..Default::default()
                },
                10
            ));
        } else {
            self.terminal
                .push(terminal_def!(terminal_name.as_str(), word, 1));
        }

        Some(vec![terminal_name])
    }

    #[inline]
    /// Transforms parenthesized expressions into a helper rule.
    fn pars(&mut self, tree: &[AST]) -> OptVecStr {
        let mut v_result: Vec<String> = Vec::new();
        let mut name = "_".to_string();

        for node in tree.iter() {
            if let Some(v_str) = self._transform(node) {
                let joined = v_str.join(" ").to_string();
                name.push_str(joined.replace(" ", "_").as_str());
                v_result.push(joined);
            }
        }

        self.insert_rules(name.as_str(), v_result, 0, None);

        Some(Vec::from([name]))
    }

    #[inline]
    /// Transforms optional expression into `(expr | empty)`.
    fn maybe(&mut self, tree: &[AST]) -> OptVecStr {
        let v_result = tree
            .iter()
            .filter_map(|x| self._transform(x))
            .flatten()
            .collect::<Vec<String>>()
            .join(" ")
            .trim()
            .to_string();
        Some(vec![v_result, "".to_string()])
    }

    /// Returns parsed priority token.
    fn priority(&mut self, tree: &[AST]) -> OptVecStr {
        self._transform(&tree[0])
    }

    /// Transforms alias production and injects alias rule.
    fn alias(&mut self, tree: &[AST]) -> OptVecStr {
        let last_node = tree.last().unwrap();
        let rule = self._transform(last_node).unwrap();

        let production = tree
            .iter()
            .take(tree.len() - 1)
            .flat_map(|x| self._transform(x).unwrap())
            .collect::<Vec<String>>()
            .join(" ")
            .trim()
            .to_string();

        self.insert_rules(rule.first().unwrap(), vec![production], 1, None);

        Some(rule)
    }

    /// Transforms regex literal node into a synthetic regex terminal.
    fn regex(&mut self, tree: &[AST]) -> OptVecStr {
        let node = tree.last().unwrap();
        let rule = self._transform(node).unwrap();
        let mut pattern = rule.first().unwrap().as_str();
        pattern = pattern.strip_prefix("/")?;

        let regex_flag_match = Regex::new(r"/[imsux]*$").unwrap();
        let captures = regex_flag_match.captures(pattern).unwrap().unwrap();
        let capture = captures.get(0).unwrap().as_str();
        let regex_flag = RegexFlag {
            i: capture.contains('i'),
            m: capture.contains('m'),
            s: capture.contains('s'),
            u: capture.contains('u'),
            x: capture.contains('x'),
        };

        pattern = pattern.strip_suffix(capture)?;

        let terminal_name = format!("__PATTERN__{}__1", pattern.to_uppercase());

        self.terminal
            .push(terminal_def!(terminal_name.as_str(), pattern, regex_flag, 0));

        Some(vec![terminal_name])
    }

    pub fn compile(&mut self, tree: Vec<&AST>) -> OptVecStr {
        let len = tree.len();
        for i in 0..len {
            let rule = tree.get(i).unwrap();
            self._transform(rule);
        }

        None
    }

    /// Dispatches AST transformation by node type.
    fn _transform(&mut self, ast: &AST) -> OptVecStr {
        match ast {
            AST::Token(token) => Some(vec![token.word().to_string()]),
            AST::Tree(name, tree) => match name.as_str() {
                "terminal" | "non_terminal" => self.terminal_non_terminal(tree),
                "expansions" | "expansion" => self.expansions(tree),
                "or_expansion" => self.or_expansion(tree),
                "op_expansion" => self.op_expansion(tree),
                "rule" => self.rule(tree),
                "range" => self.range(tree),
                "string" => self.string(tree),
                "alias" => self.alias(tree),
                "pars" => self.pars(tree),
                "maybe" => self.maybe(tree),
                "priority" => self.priority(tree),
                "regex" => self.regex(tree),
                _ => panic!("not matched, {name}"),
            },
        }
    }
}


