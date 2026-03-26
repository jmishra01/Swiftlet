use crate::grammar::{Rule, RuleOption};
use crate::lexer::{RegexFlag, Symbol, TerminalDef, get_symbol};
use crate::ast::AST;
use crate::{terminal_def};
use fancy_regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

pub type OptVecStr = Option<Vec<String>>;

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
pub struct Transformer {
    terminal: Vec<Arc<TerminalDef>>,
    ignores: Vec<String>,
    common_terminals: HashMap<String, Arc<TerminalDef>>,
    count: usize,
    cache: HashMap<String, String>,
    rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>,
}

impl Transformer {
    /// Creates an empty transformer with shared built-in terminals.
    pub fn new(common_terminals: HashMap<String, Arc<TerminalDef>>) -> Self {
        Self {
            terminal: Vec::new(),
            ignores: Vec::new(),
            common_terminals,
            count: 0usize,
            cache: HashMap::new(),
            rules: HashMap::new(),
        }
    }

    /// Returns transformed grammar and validates non-terminal references.
    pub fn get_grammar(&self) -> HashMap<Arc<Symbol>, Vec<Arc<Rule>>> {
        for v in self.rules.values() {
            for r in v.iter() {
                for e in r.expansion.iter() {
                    if !e.is_terminal() && !self.rules.contains_key(e) {
                        panic!(
                            "\"{}\" not defined in the given grammar",
                            e.as_ref().as_str()
                        );
                    }
                }
            }
        }

        self.rules.clone()
    }

    /// Sorts terminals by descending max width for longest-match behavior.
    pub(crate) fn sort_terminals(&mut self) {
        self.terminal.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(
                    b.max_width.cmp(&a.max_width)
                        .then(b.value.len().cmp(&a.value.len())))
        }
        );
    }

    /// Returns generated terminals.
    pub fn get_terminal(&self) -> Vec<Arc<TerminalDef>> {
        self.terminal.clone()
    }
    /// Returns collected ignore directives.
    pub fn get_ignores(&self) -> Vec<String> {
        self.ignores.clone()
    }

    /// Prints transformed grammar rules.
    #[cfg(feature = "debug")]
    pub fn print_grammar(&self) {
        println!("\nGrammar");
        println!("=======");
        for (_, prod) in self.rules.iter() {
            for p in prod.iter() {
                println!("{p:?}");
            }
        }
    }

    /// Prints transformed terminals.
    #[cfg(feature = "debug")]
    pub fn print_terminals(&self) {
        println!("\nTerminals");
        println!("=========");
        for t in self.terminal.iter() {
            println!("{t:?}");
        }
    }

    #[inline]
    /// Transforms a terminal or non-terminal leaf node.
    fn terminal_non_terminal(&mut self, tree: &[AST]) -> OptVecStr {
        self.transform(&tree[0])
    }

    #[inline]
    /// Expands concatenated expressions into full production combinations.
    fn expansions(&mut self, tree: &[AST]) -> OptVecStr {
        if tree.len() == 1 {
            let ret = self.transform(&tree[0]);
            return ret;
        }
        let mut v_result = Vec::from(["".to_string()]);
        for node in tree.iter() {
            if let Some(t) = self.transform(node) {
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
            return self.transform(tree.first().unwrap());
        }

        let result: Vec<String> = tree
            .iter()
            .filter_map(|x| self.transform(x))
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
        let second = self.transform(&tree[1]).unwrap()[0].clone();

        if let AST::Tree(name, _) = &tree[0]
            && name.cmp(&"terminal".to_string()).is_eq()
        {
            let v_result = self.transform(&tree[0]).unwrap()[0].clone();

            if let Some(r) = self
                .terminal
                .iter()
                .find(|&x| x.get_name().as_ref().as_str() == v_result.as_str())
            {
                return Some(vec![format!("{}{}", r.value, second).trim().to_string()]);
            }
        }

        let mut v_result = Vec::from([self.transform(&tree[0]).unwrap().join(" ")]);

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
        let rule_name = self.transform(&tree[0]).unwrap();
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


        let prod = self.transform(child).unwrap();
        let priority = if tree.len() > 2 {
            let priority = self.transform(&tree[1]).unwrap();
            priority[0].as_str().parse::<usize>().unwrap()
        } else {
            0
        };

        self.insert_rules(rule_name[0].as_str(), prod, priority, Some(alias_rules));

        None
    }

    #[inline]
    /// Transforms a terminal declaration node.
    fn term(&mut self, tree: &[AST]) -> OptVecStr {
        let first = tree[0].clone();
        let second = tree[1].clone();
        let term_name = self.transform(&first);
        let prod = self.transform(&second).unwrap();
        let prod_sym: Arc<Symbol> = Arc::new(Symbol::Terminal(prod.first().unwrap().to_string()));
        let priority = if second.is_tree_exist("regex") { 5 } else { 10 };

        if let Some(index) = self.terminal.iter().position(|x| x.get_name() == prod_sym) {
            let val = self.terminal.remove(index);
            self.terminal.push(Arc::new(TerminalDef {
                name: Arc::new(Symbol::Terminal(
                    term_name.unwrap().first().unwrap().clone(),
                )),
                value: val.value.clone(),
                pattern: val.pattern.clone(),
                max_width: val.max_width,
                priority,
            }));
        } else {
            // Else used to transform below grammar pattern
            /*
                start: NAME
                NAME: CNAME
                %import (CNAME)
             */
            self.terminal.push(terminal_def!(
                term_name.unwrap().first().unwrap(),
                prod.first().unwrap(),
                RegexFlag::default(),
                10
            ));
        }

        None
    }

    #[inline]
    /// Transforms a string range node into a character class regex.
    fn range(&mut self, tree: &[AST]) -> OptVecStr {
        let first = self.transform(&tree[0]).unwrap();
        let second = self.transform(&tree[1]).unwrap();

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
        let word = self.transform(&tree[0]).unwrap();
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
    /// Processes root start node in two passes: terminals first, then rules.
    fn start(&mut self, tree: &[AST]) -> OptVecStr {
        for node in tree.iter() {
            if let Some(name) = node.get_tree_name()
                && name.cmp(&"terminal".to_string()).is_eq()
            {
                self.transform(node);
            }
        }
        for node in tree.iter() {
            if let Some(name) = node.get_tree_name()
                && name.cmp(&"terminal".to_string()).is_ne()
            {
                self.transform(node);
            }
        }
        None
    }

    #[inline]
    /// Collects ignore terminal names from ignore directives.
    fn ignore(&mut self, tree: &[AST]) -> OptVecStr {
        for node in tree.iter() {
            if let Some(v) = self.transform(node) {
                for i in v.iter() {
                    self.ignores.push(i.clone());
                }
            }
        }
        None
    }

    #[inline]
    /// Transforms parenthesized expressions into a helper rule.
    fn pars(&mut self, tree: &[AST]) -> OptVecStr {
        let mut v_result: Vec<String> = Vec::new();
        let mut name = "_".to_string();

        for node in tree.iter() {
            if let Some(v_str) = self.transform(node) {
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
            .filter_map(|x| self.transform(x))
            .flatten()
            .collect::<Vec<String>>()
            .join(" ")
            .trim()
            .to_string();
        Some(vec![v_result, "".to_string()])
    }

    /// Imports requested common terminals into the current grammar.
    fn import(&mut self, tree: &[AST]) -> OptVecStr {
        for x in tree.iter(){
            if let Some(words) = self.transform(x) {
                for w in words.iter() {
                    if self.common_terminals.contains_key(w) {
                        self.terminal.push(self.common_terminals.get(w).unwrap().clone());
                    }
                }
            }
        }
        None
    }

    /// Returns parsed priority token.
    fn priority(&mut self, tree: &[AST]) -> OptVecStr {
        self.transform(&tree[0])
    }

    /// Transforms alias production and injects alias rule.
    fn alias(&mut self, tree: &[AST]) -> OptVecStr {
        let last_node = tree.last().unwrap();
        let rule = self.transform(last_node).unwrap();

        let production = tree
            .iter()
            .take(tree.len() - 1)
            .flat_map(|x| self.transform(x).unwrap())
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
        let rule = self.transform(node).unwrap();
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

    /// Dispatches AST transformation by node type.
    pub fn transform(&mut self, ast: &AST) -> OptVecStr {
        match ast {
            AST::Token(token) => Some(vec![token.word().to_string()]),
            AST::Tree(name, tree) => match name.as_str() {
                "terminal" | "non_terminal" => self.terminal_non_terminal(tree),
                "expansions" | "expansion" => self.expansions(tree),
                "or_expansion" => self.or_expansion(tree),
                "rule" => self.rule(tree),
                "term" => self.term(tree),
                "range" => self.range(tree),
                "string" => self.string(tree),
                "start" => self.start(tree),
                "alias" => self.alias(tree),
                "ignore" => self.ignore(tree),
                "pars" => self.pars(tree),
                "maybe" => self.maybe(tree),
                "priority" => self.priority(tree),
                "op_expansion" => self.op_expansion(tree),
                "import" | "name_list" => self.import(tree),
                "regex" => self.regex(tree),
                _ => panic!("not matched, {name}"),
            },
        }
    }
}
