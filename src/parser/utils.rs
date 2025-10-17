use crate::grammar::Rule;
use std::sync::Arc;

pub fn dot_state(rule: &Arc<Rule>, dot: usize) -> (String, String, String) {
    let origin = rule.origin.get_value();

    let mut before_dot = "".to_string();
    let mut after_dot = "".to_string();

    for (index, prod) in rule.expansion.iter().enumerate() {
        if index < dot {
            before_dot = format!("{} {}", before_dot, prod.get_value());
        }
        if index >= dot {
            after_dot = format!("{} {}", after_dot, prod.get_value());
        }
    }

    (origin, before_dot, after_dot)
}

#[cfg(test)]
mod tests {
    use crate::grammar::{Rule, RuleOption};
    use crate::lexer::Symbol;
    use crate::parser::utils::dot_state;
    use crate::{non_terms, terms};
    use std::sync::Arc;
    use std::sync::LazyLock;

    static RULES: LazyLock<Arc<Rule>> = LazyLock::new(|| {
        let rules = Arc::new(Rule::new(
            non_terms!("expr".to_string()),
            vec![non_terms!("expr"), terms!("id")],
            Arc::from(RuleOption::default()),
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
