use crate::grammar::Rule;
use std::sync::Arc;

/// Returns `(origin, before_dot, after_dot)` for visualizing parser item state.
pub fn dot_state(rule: &Arc<Rule>, dot: usize) -> (String, String, String) {
    let origin = rule.origin.as_ref().as_str().to_string();

    let mut before_dot = "".to_string();
    let mut after_dot = "".to_string();

    for (index, prod) in rule.expansion.iter().enumerate() {
        if index < dot {
            before_dot = format!("{} {}", before_dot, prod.as_ref().as_str());
        }
        if index >= dot {
            after_dot = format!("{} {}", after_dot, prod.as_ref().as_str());
        }
    }

    (origin, before_dot, after_dot)
}

#[cfg(test)]
mod tests {
    use crate::grammar::{Rule, RuleMeta};
    use crate::lexer::Symbol;
    use crate::parser::utils::dot_state;
    use crate::{non_terms, terms};
    use std::sync::Arc;
    use std::sync::LazyLock;

    static RULES: LazyLock<Arc<Rule>> = LazyLock::new(|| {
        let rules = Arc::new(Rule::new(
            non_terms!("expr".to_string()),
            vec![non_terms!("expr"), terms!("id")],
            Arc::from(RuleMeta::default()),
            0,
        ));
        rules
    });

    #[test]
    fn test_dot_state_dot_at_zero() {
        let left = dot_state(&RULES, 0);
        let right = ("expr".to_string(), "".to_string(), " expr id".to_string());
        assert_eq!(left, right);
    }

    #[test]
    fn test_dot_state_dot_at_one() {
        let left = dot_state(&RULES, 1);
        let right = ("expr".to_string(), " expr".to_string(), " id".to_string());
        assert_eq!(left, right);
    }

    #[test]
    fn test_dot_state_dot_at_last() {
        let left = dot_state(&RULES, 2);
        let right = ("expr".to_string(), " expr id".to_string(), "".to_string());
        assert_eq!(left, right);
    }
}
