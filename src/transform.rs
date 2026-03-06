use crate::grammar::{Rule, RuleOption};
use crate::lexer::{get_symbol, RegexFlag, Symbol, TerminalDef, AST};
use crate::{terminal_def, terms};
use std::collections::HashMap;
use std::sync::Arc;
use fancy_regex::Regex;

pub type OVecStr = Option<Vec<String>>;

fn origin_apply(name: &str, priority: usize) -> (String, Arc<RuleOption>) {
    let is_expand = name.starts_with("?");
    let rule_option = Arc::new(RuleOption::new(is_expand, priority));
    let clean_name = if is_expand {
        name.strip_prefix("?").unwrap().to_string()
    } else {
        name.to_string()
    };

    (clean_name, rule_option)
}

pub struct Transformer {
    terminal: Vec<Arc<TerminalDef>>,
    ignores: Vec<String>,
    common_terminals: HashMap<String, Arc<TerminalDef>>,
    count: usize,
    cache: HashMap<String, String>,
    rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>>,
}

impl Transformer {
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

    pub fn get_grammar(&self) -> HashMap<Arc<Symbol>, Vec<Arc<Rule>>> {
        for v in self.rules.values() {
            for r in v.iter() {
                for e in r.expansion.iter() {
                    if !e.is_terminal() && !self.rules.contains_key(e) {
                        panic!("\"{}\" not defined in the given grammar", e.get_value());
                    }
                }
            }
        }

        self.rules.clone()
    }

    pub(crate) fn sort_terminals(&mut self) {
        self.terminal.sort_by(|a, b| b.max_width.cmp(&a.max_width));
    }

    pub fn get_terminal(&self) -> Vec<Arc<TerminalDef>> {
        self.terminal.clone()
    }
    pub fn get_ignores(&self) -> Vec<String> {
        self.ignores.clone()
    }

    #[cfg(feature = "debug")]
    pub fn print_grammar(&self) {
        println!("Grammar");
        for (_, prod) in self.rules.iter() {
            for p in prod.iter() {
                println!("{p:?}");
            }
        }
    }

    #[cfg(feature = "debug")]
    pub fn print_terminals(&self) {
        println!("Terminals");
        for t in self.terminal.iter() {
            println!("{t:?}");
        }
    }

    #[inline]
    fn terminal_non_terminal(&mut self, tree: &[AST]) -> OVecStr {
        self.transform(&tree[0])
    }

    #[inline]
    fn expansions(&mut self, tree: &[AST]) -> OVecStr {
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
    fn or_expansion(&mut self, tree: &[AST]) -> OVecStr {
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
        self.insert_rules(rule_name.as_str(), result, 0);

        Some(vec![rule_name])
    }

    #[inline]
    fn op_expansion(&mut self, tree: &[AST]) -> OVecStr {
        let second = self.transform(&tree[1]).unwrap()[0].clone();

        if let AST::Tree(name, _) = &tree[0]
            && name.cmp(&"terminal".to_string()).is_eq()
        {
            let v_result = self.transform(&tree[0]).unwrap()[0].clone();

            if let Some(r) = self
                .terminal
                .iter()
                .find(|&x| x.get_name().get_value().cmp(&v_result).is_eq())
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

                self.insert_rules(r.as_str(), vec![format!("{} {}", r, e), e.to_string()], 0);

                match second {
                    "+" => Some(vec![r]),
                    _ => Some(vec![r, "".to_string()]),
                }
            }
            _ => None,
        }
    }

    fn insert_rules(&mut self, rule_name: &str, prod: Vec<String>, priority: usize) {
        let (clean_name, rule_option) = origin_apply(rule_name, priority);

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
    fn rule(&mut self, tree: &[AST]) -> OVecStr {
        let rule_name = self.transform(&tree[0]).unwrap();
        let prod = self.transform(tree.last().unwrap()).unwrap();

        let priority = if tree.len() > 2 {
            let priority = self.transform(&tree[1]).unwrap();
            priority[0].as_str().parse::<usize>().unwrap()
        } else {
            0
        };

        self.insert_rules(rule_name[0].as_str(), prod, priority);

        None
    }

    #[inline]
    fn term(&mut self, tree: &[AST]) -> OVecStr {
        let first = tree[0].clone();
        let second = tree[1].clone();
        let rule_name = self.transform(&first);
        let prod = self.transform(&second).unwrap();

        let prod_sym: Arc<Symbol> = terms!(prod.first().unwrap());

        if let Some(index) = self.terminal.iter().position(|x| x.get_name() == prod_sym) {
            let val = self.terminal.remove(index);
            self.terminal.push(Arc::new(TerminalDef {
                name: Arc::new(Symbol::Terminal(
                    rule_name.unwrap().first().unwrap().clone(),
                )),
                value: val.value.clone(),
                pattern: val.pattern.clone(),
                max_width: val.max_width,
            }));
        } else {
            self.terminal.push(terminal_def!(
                rule_name.unwrap().first().unwrap(),
                prod.first().unwrap(),
                RegexFlag::default()
            ));
        }

        None
    }

    #[inline]
    fn range(&mut self, tree: &[AST]) -> OVecStr {
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
    fn string(&mut self, tree: &[AST]) -> OVecStr {
        let word = self.transform(&tree[0]).unwrap();
        let word = word[0].strip_prefix("\"")?;
        let is_case_insensitive = word.ends_with("\"i");
        let word = if is_case_insensitive { word.strip_suffix("\"i")? } else { word.strip_suffix("\"")? };
        let terminal_name = format!("__STR__{}__1", word.to_uppercase());

        if is_case_insensitive {
            self.terminal
                .push(terminal_def!(terminal_name.as_str(), word, RegexFlag {i: is_case_insensitive, ..Default::default()}));
        } else {
            self.terminal
                .push(terminal_def!(terminal_name.as_str(), word));
        }

        Some(vec![terminal_name])
    }

    #[inline]
    fn start(&mut self, tree: &[AST]) -> OVecStr {
        for node in tree.iter() {
            if let Some(name) = node.get_tree_name()
                && name.cmp(&"terminal".to_string()).is_eq() {
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
    fn ignore(&mut self, tree: &[AST]) -> OVecStr {
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
    fn pars(&mut self, tree: &[AST]) -> OVecStr {
        let mut v_result: Vec<String> = Vec::new();
        let mut name = "_".to_string();

        for node in tree.iter() {
            if let Some(v_str) = self.transform(node) {
                let joined = v_str.join(" ").to_string();
                name.push_str(joined.replace(" ", "_").as_str());
                v_result.push(joined);
            }
        }

        self.insert_rules(name.as_str(), v_result, 0);

        Some(Vec::from([name]))
    }

    #[inline]
    fn maybe(&mut self, tree: &[AST]) -> OVecStr {
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

    fn import(&mut self, tree: &[AST]) -> OVecStr {
        for x in tree.iter() {
            if let Some(words) = self.transform(x) {
                for w in words.iter() {
                    if self.common_terminals.contains_key(w) {
                        self.terminal
                            .push(self.common_terminals.get(w).unwrap().clone());
                    }
                }
            }
        }
        None
    }

    fn priority(&mut self, tree: &[AST]) -> OVecStr {
        self.transform(&tree[0])
    }

    fn alias(&mut self, tree: &[AST]) -> OVecStr {
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

        self.insert_rules(rule.first().unwrap(), vec![production], 1);

        Some(rule)
    }

    fn regex(&mut self, tree: &[AST]) -> OVecStr {
        let node = tree.last().unwrap();
        let rule = self.transform(node).unwrap();
        let mut pattern = rule.first().unwrap().as_str();
        pattern = pattern.strip_prefix("/")?;

        let regex_flag_match = Regex::new(r"/[imsux]*$").unwrap();

        let captures = regex_flag_match.captures(pattern).unwrap().unwrap();

        let capture = captures.get(1).unwrap().as_str();

        pattern = pattern.strip_suffix(capture)?;

        let regex_flag = RegexFlag {
            i: capture.contains('i'),
            m: capture.contains('m'),
            s: capture.contains('s'),
            u: capture.contains('u'),
            x: capture.contains('x'),
            };

        let terminal_name = format!("__PATTERN__{}__1", pattern.to_uppercase());

        self.terminal.push(terminal_def!(terminal_name.as_str(), pattern, regex_flag));

        Some(vec![terminal_name])
    }

    pub fn transform(&mut self, ast: &AST) -> OVecStr {
        match ast {
            AST::Token(token) => Some(vec![token.word.clone()]),
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
