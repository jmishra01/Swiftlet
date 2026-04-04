use crate::lexer::{Symbol, get_symbol};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

/// Selects the parsing algorithm used to build the parser.
#[derive(Clone, Debug)]
pub enum Algorithm {
    Earley,
    CLR,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub(crate) struct RuleOption {
    expand: bool,
    priority: usize,
    alias_rule: Option<Vec<String>>,
}

impl RuleOption {
    /// Creates rule metadata with expand flag and precedence priority.
    pub(crate) fn new(expand: bool, priority: usize, alias_rule: Option<Vec<String>>) -> Self {
        Self {
            expand,
            priority,
            alias_rule,
        }
    }

    /// Returns whether this rule should be expanded (flattened) in tree building.
    pub(crate) fn is_expand(&self) -> bool {
        self.expand
    }

    /// Returns rule priority used for conflict resolution.
    pub(crate) fn priority(&self) -> usize {
        self.priority
    }

    pub(crate) fn alias_rule(&self) -> Option<&[String]> {
        self.alias_rule.as_deref()
    }
}

/// Represents a single grammar production.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Rule {
    pub(crate) origin: Arc<Symbol>,
    pub(crate) expansion: Vec<Arc<Symbol>>,
    pub(crate) rule_option: Arc<RuleOption>,
    pub(crate) order: usize,
}

impl Rule {
    /// Creates a grammar rule with a cached expansion length.
    pub(crate) fn new(
        origin: Arc<Symbol>,
        expansion: Vec<Arc<Symbol>>,
        rule_option: Arc<RuleOption>,
        order: usize,
    ) -> Self {
        Self {
            origin,
            expansion,
            rule_option,
            order,
        }
    }

    /// Returns whether this rule should be expanded in the resulting AST.
    pub fn is_expand(&self) -> bool {
        self.rule_option.expand
    }

    /// Returns the expansion length.
    pub(crate) const fn len(&self) -> usize {
        self.expansion.len()
    }
}

impl Display for Rule {
    /// Formats rule as `origin -> expansion...`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lhs = self.origin.as_ref().as_str();
        let rhs = self
            .expansion
            .iter()
            .map(|x| x.as_ref().as_str().to_string())
            .collect::<Vec<_>>();

        write!(f, "{} -> {}", lhs, rhs.join(" "))
    }
}

/// Builds a rule map from `(origin, expansions)` tuples.
///
/// Rule names prefixed with `?` are marked as expandable.
pub fn create_rules<S, T>(arr: T) -> HashMap<Arc<Symbol>, Vec<Arc<Rule>>>
where
    S: AsRef<str>,
    T: IntoIterator<Item = (S, Vec<S>)>,
{
    let rules: HashMap<Arc<Symbol>, Vec<Arc<Rule>>> =
        HashMap::from_iter(arr.into_iter().map(|(name, expansions)| {
            let name = name.as_ref();
            let is_expand = name.starts_with("?");
            let rule_option = Arc::new(RuleOption::new(is_expand, 0, None));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_option_new_and_accessors_work() {
        let opts = RuleOption::new(true, 7, None);
        assert!(opts.is_expand());
        assert_eq!(opts.priority(), 7);
    }

    #[test]
    fn rule_new_len_debug_and_expand_work() {
        let rule = Rule::new(
            Arc::new(Symbol::NonTerminal("start".to_string())),
            vec![
                Arc::new(Symbol::NonTerminal("expr".to_string())),
                Arc::new(Symbol::Terminal("INT".to_string())),
            ],
            Arc::new(RuleOption::new(false, 0, None)),
            1,
        );

        assert_eq!(rule.len(), 2);
        assert!(!rule.is_expand());
        assert_eq!(format!("{rule}"), "start -> expr INT");
    }

    #[test]
    fn create_rules_builds_non_terminals_and_expansions() {
        let rules = create_rules([
            ("start", vec!["expr"]),
            ("?expr", vec!["INT", "expr PLUS INT"]),
        ]);

        let start = Arc::new(Symbol::NonTerminal("start".to_string()));
        let expr = Arc::new(Symbol::NonTerminal("expr".to_string()));

        assert!(rules.contains_key(&start));
        assert!(rules.contains_key(&expr));
        assert_eq!(rules[&start].len(), 1);
        assert_eq!(rules[&expr].len(), 2);
        assert!(rules[&expr][0].is_expand());
    }
}
