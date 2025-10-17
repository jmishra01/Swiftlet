use crate::lexer::{Symbol, get_symbol};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Algorithm {
    Earley,
    CLR,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub(crate) struct RuleOption {
    expand: bool,
    priority: usize,
}

impl RuleOption {
    pub(crate) fn new(expand: bool, priority: usize) -> Self {
        Self { expand, priority }
    }

    pub(crate) fn is_expand(&self) -> bool {
        self.expand
    }

    pub(crate) fn priority(&self) -> usize {
        self.priority
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Rule {
    pub(crate) origin: Arc<Symbol>,
    pub(crate) expansion: Vec<Arc<Symbol>>,
    pub(crate) rule_option: Arc<RuleOption>,
    pub(crate) order: usize,
    pub(crate) expansion_len: usize,
}

impl Rule {
    pub(crate) fn new(
        origin: Arc<Symbol>,
        expansion: Vec<Arc<Symbol>>,
        rule_option: Arc<RuleOption>,
        order: usize,
    ) -> Self {
        let expansion_len = expansion.len();
        Self {
            origin,
            expansion,
            rule_option,
            order,
            expansion_len,
        }
    }

    pub fn is_expand(&self) -> bool {
        self.rule_option.expand
    }

    pub(crate) const fn len(&self) -> usize {
        self.expansion_len
    }
}

impl Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lhs = self.origin.get_value();
        let rhs = self
            .expansion
            .iter()
            .map(|x| x.get_value())
            .collect::<Vec<_>>();

        write!(f, "{} -> {}", lhs, rhs.join(" "))
    }
}

pub fn create_rules<S, T>(arr: T) -> HashMap<Arc<Symbol>, Vec<Arc<Rule>>>
where
    S: AsRef<str>,
    T: IntoIterator<Item = (S, Vec<S>)>,
{
    let rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>> =
        HashMap::from_iter(arr.into_iter().map(|(name, expansions)| {
            let name = name.as_ref();
            let is_expand = name.starts_with("?");
            let rule_option = Arc::new(RuleOption::new(is_expand, 0));
            let clean_name = if is_expand {
                name.strip_prefix("?").unwrap().to_string()
            } else {
                name.to_string()
            };

            (
                get_symbol(clean_name.as_str()),
                expansions
                    .iter()
                    .enumerate()
                    .map(|(order, expansion)| {
                        Arc::new(Rule::new(
                            Arc::new(Symbol::NonTerminal(clean_name.clone())),
                            expansion.as_ref().split(" ").map(get_symbol).collect(),
                            rule_option.clone(),
                            order,
                        ))
                    })
                    .collect(),
            )
        }));
    rules
}
