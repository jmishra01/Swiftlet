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

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Earley => write!(f, "Earley"),
            Algorithm::CLR => write!(f, "CLR"),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub(crate) struct RuleMeta {
    expand: bool,
    priority: usize,
    alias_rule: Option<Vec<String>>,
}

impl RuleMeta {
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

    // Returns the list of alias names attached to this rule, or `None` if no aliases were declared.
    pub(crate) fn alias_rule(&self) -> Option<&[String]> {
        self.alias_rule.as_deref()
    }
}

/// Represents a single grammar production.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Rule {
    pub(crate) origin: Arc<Symbol>,
    pub(crate) expansion: Vec<Arc<Symbol>>,
    pub(crate) rule_option: Arc<RuleMeta>,
    pub(crate) order: usize,
    /// Precomputed: origin name starts with `_` (hidden node -- children are inline).
    pub(crate) is_hidden: bool,
    /// Precomputed: rule should be expanded (flattened) in the AST.
    pub(crate) expand: bool
}

impl Rule {
    /// Creates a grammar rule with a cached `is_hidden` and `expand` flags
    pub(crate) fn new(
        origin: Arc<Symbol>,
        expansion: Vec<Arc<Symbol>>,
        rule_option: Arc<RuleMeta>,
        order: usize,
    ) -> Self {
        let is_hidden = origin.as_str().starts_with('_');
        let expand = rule_option.expand;
        Self {
            origin,
            expansion,
            rule_option,
            order,
            is_hidden,
            expand
        }
    }

    /// Returns whether this rule should be expanded (flattened) in the resulting AST.
    #[inline(always)]
    pub fn is_expand(&self) -> bool {
        self.expand
    }

    /// Returns the non-terminal symbol this rule reduces to (the left-hand side).
    pub fn origin(&self) -> &Symbol {
        &self.origin
    }

    // Returns the right-hand side symbols of this production.
    pub fn expansion(&self) -> &[Arc<Symbol>] {
        &self.expansion
    }

    /// Returns the rule priority used for conflict resolution
    pub fn priority(&self) -> usize {
        self.rule_option.priority()
    }

    /// Returns the expansion length.
    pub(crate) const fn len(&self) -> usize {
        self.expansion.len()
    }
}

impl Display for Rule {
    /// Formats rule as `origin -> expansion...`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ->", self.origin.as_ref().as_str())?;
        for sym in &self.expansion {
            write!(f, " {}", sym.as_ref().as_str())?;
        }
        Ok(())
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
            let rule_option = Arc::new(RuleMeta::new(is_expand, 0, None));
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
        let opts = RuleMeta::new(true, 7, None);
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
            Arc::new(RuleMeta::new(false, 0, None)),
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

    #[test]
    fn algorithm_display_works_for_both_variants() {
        assert_eq!(format!("{}", Algorithm::Earley), "Earley");
        assert_eq!(format!("{}", Algorithm::CLR), "CLR");
    }

    #[test]
    fn rule_origin_expansion_and_priority_accessors_work() {
        let origin = Arc::new(Symbol::NonTerminal("start".to_string()));
        let sym = Arc::new(Symbol::Terminal("INT".to_string()));
        let rule = Rule::new(
            origin.clone(),
            vec![sym.clone()],
            Arc::new(RuleMeta::new(false, 5, None)),
            0,
        );

        assert_eq!(rule.origin().as_str(), "start");
        assert_eq!(rule.expansion().len(), 1);
        assert_eq!(rule.expansion()[0].as_str(), "INT");
        assert_eq!(rule.priority(), 5);
    }

    #[test]
    fn rule_meta_alias_rule_accessor_works() {
        let aliases = vec!["add".to_string(), "sub".to_string()];
        let meta = RuleMeta::new(false, 0, Some(aliases.clone()));
        assert_eq!(meta.alias_rule(), Some(aliases.as_slice()));

        let no_alias = RuleMeta::new(false, 0, None);
        assert_eq!(no_alias.alias_rule(), None);
    }
}
