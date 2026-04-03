use crate::lexer::Token;
use std::sync::Arc;

/// Represents either a token leaf or a named tree node in the parse result.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AST {
    Token(Arc<Token>),
    Tree(String, Vec<AST>),
}

impl AST {
    /// Returns the tree node name for `AST::Tree`, otherwise `None`.
    pub fn get_tree_name(&self) -> Option<&String> {
        match self {
            AST::Tree(name, _) => Some(name),
            _ => unreachable!(),
        }
    }

    /// Checks whether this AST node should be flattened by underscore naming rules.
    pub fn is_start_with_underscore(&self) -> bool {
        match self {
            AST::Token(token) => {
                token.terminal.as_ref().as_str().starts_with("_")
                    && !token.terminal.as_ref().as_str().starts_with("__")
            }
            AST::Tree(name, _) => name.starts_with("_") && !name.starts_with("__"),
        }
    }

    /// Prints a multi-line pretty representation of the AST.
    pub fn pretty_print(&self) {
        pretty_print(self, "".to_string());
    }

    /// Prints a single-line AST representation.
    pub fn print(&self) {
        println!("{}", self.get_text());
    }

    /// Return true if child tree exist
    pub fn is_tree_exist(&self, tree_name: &str) -> bool {
        match self {
            AST::Token(_) => false,
            AST::Tree(name, children) => {
                if *name == tree_name {
                    return true;
                }
                if children.is_empty() {
                    return false;
                }
                children.iter().any(|child| child.is_tree_exist(tree_name))
            }
        }
    }

    pub fn get_children(&self) -> Option<&Vec<AST>> {
        if let AST::Tree(_, children) = self {
            return Some(children);
        }
        None
    }

    /// Returns a single-line AST representation.
    pub fn get_text(&self) -> String {
        inline_print(self)
    }
    // Original — finds first match only
    /// Returns the first subtree with the provided name.
    pub fn get_tree<'a>(&'a self, tree_name: &'a str) -> Option<&'a AST> {
        self.iter_trees(tree_name).next()
    }

    /// Lazily iterates over every subtree with the provided name.
    pub fn iter_trees<'a>(&'a self, tree_name: &'a str) -> ASTTreeIter<'a> {
        ASTTreeIter::new(self, tree_name)
    }

    /// Returns the children collections for all matching subtrees.
    pub fn get_child_tree(&self, tree_name: &str) -> Option<Vec<&AST>> {
        match self {
            AST::Token(_) => None,
            AST::Tree(name, children) => {
                let mut ast_vec = Vec::new();
                if name == tree_name {
                    ast_vec.push(self);
                }
                for child in children {
                    if let Some(rule) = child.get_child_tree(tree_name) {
                        ast_vec.extend(rule);
                    }
                }
                Some(ast_vec)
            }
        }
    }

    pub fn get_last_child(&self) -> Option<&AST> {
        match self {
            AST::Token(_) => None,
            AST::Tree(_, children) => children.last(),
        }
    }
}

/// Depth-first iterator over matching AST tree nodes.
pub struct ASTTreeIter<'a> {
    stack: Vec<&'a AST>,
    tree_name: &'a str,
}

impl<'a> ASTTreeIter<'a> {
    fn new(root: &'a AST, tree_name: &'a str) -> Self {
        ASTTreeIter {
            stack: vec![root],
            tree_name,
        }
    }
}

impl<'a> Iterator for ASTTreeIter<'a> {
    type Item = &'a AST;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if let AST::Tree(name, children) = node {
                // Push children onto stack for future traversal (reversed for left-to-right order)
                self.stack.extend(children.iter().rev());

                if name == self.tree_name {
                    return Some(node); // yield this match
                }
            }
        }
        None // stack exhausted
    }
}

/// Converts AST to a compact single-line textual form.
fn inline_print(tree: &AST) -> String {
    match tree {
        AST::Token(token) => {
            let word = token.word().to_string();
            let terminal = token.terminal.get_value();
            if terminal.starts_with("__") {
                format!("{:?}", word)
            } else {
                format!(
                    "Token({}, \"{}\")",
                    token.terminal.get_value(),
                    token.word()
                )
            }
        }
        AST::Tree(name, children) => {
            let c = children
                .iter()
                .map(inline_print)
                .collect::<Vec<String>>()
                .join(", ");
            format!("Tree(\"{}\", [{}])", name, c)
        }
    }
}

/// Recursively pretty-prints an AST with indentation padding.
fn pretty_print(tree: &AST, space: String) {
    match tree {
        AST::Token(name) => println!("{}{}", space, name.word()),
        AST::Tree(name, v_ast) => {
            if v_ast.len() == 1 && matches!(v_ast[0], AST::Token(_)) {
                let val = match &v_ast[0] {
                    AST::Token(token) => token.word(),
                    _ => panic!("Not a token"),
                };
                println!("{}{}  {}", space, name, val);
                return;
            }

            println!("{}{}", space, name);
            let _rep = " ".to_string().repeat(name.len().div_ceil(2));
            for _ast in v_ast {
                pretty_print(_ast, space.clone() + "  ");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Symbol;

    fn token_ast(word: &str, terminal: &str) -> AST {
        AST::Token(Arc::new(Token::new(
            Arc::<str>::from(word),
            0,
            word.len(),
            0,
            Arc::new(Symbol::Terminal(terminal.to_string())),
        )))
    }

    fn sample_tree() -> AST {
        AST::Tree(
            "root".to_string(),
            vec![
                AST::Tree("expr".to_string(), vec![token_ast("a", "IDENT")]),
                AST::Tree(
                    "_hidden".to_string(),
                    vec![
                        AST::Tree("expr".to_string(), vec![token_ast("b", "IDENT")]),
                        AST::Tree("leaf".to_string(), vec![token_ast("c", "__RAW")]),
                    ],
                ),
            ],
        )
    }

    #[test]
    fn ast_struct_and_basic_helpers_work() {
        let ast_tok = token_ast("hello", "_WS");
        let ast_tree = AST::Tree("node".to_string(), vec![ast_tok.clone()]);

        assert_eq!(ast_tree.get_tree_name(), Some(&"node".to_string()));
        assert!(ast_tok.is_start_with_underscore());
        assert!(!ast_tree.is_start_with_underscore());
        assert_eq!(ast_tree.get_children().map(Vec::len), Some(1));
        assert_eq!(ast_tok.get_children(), None);
        assert_eq!(ast_tree.get_last_child(), Some(&ast_tok));
        assert_eq!(ast_tok.get_last_child(), None);
        assert!(ast_tree.get_text().starts_with("Tree(\"node\""));
    }

    #[test]
    fn tree_search_helpers_find_expected_nodes() {
        let tree = sample_tree();

        assert!(tree.is_tree_exist("expr"));
        assert!(tree.is_tree_exist("leaf"));
        assert!(!tree.is_tree_exist("missing"));

        let first_expr = tree.get_tree("expr");
        assert!(matches!(first_expr, Some(AST::Tree(name, _)) if name == "expr"));

        let expr_nodes = tree
            .get_child_tree("expr")
            .expect("expr nodes should exist");
        assert_eq!(expr_nodes.len(), 2);

        let leaf_nodes = tree
            .get_child_tree("leaf")
            .expect("leaf nodes should exist");
        assert_eq!(leaf_nodes.len(), 1);

        assert!(matches!(tree.get_tree("missing"), None));
        assert_eq!(token_ast("x", "IDENT").get_child_tree("expr"), None);
    }

    #[test]
    fn iter_trees_yields_matches_in_depth_first_left_to_right_order() {
        let tree = sample_tree();

        let names = tree
            .iter_trees("expr")
            .map(|ast| ast.get_tree_name().expect("tree nodes only").clone())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["expr".to_string(), "expr".to_string()]);
    }

    #[test]
    fn inline_print_formats_tokens_and_trees() {
        let regular_token = token_ast("abc", "IDENT");
        let hidden_token = token_ast(" ", "__WS");
        let tree = AST::Tree(
            "pair".to_string(),
            vec![regular_token.clone(), hidden_token.clone()],
        );

        assert_eq!(inline_print(&regular_token), r#"Token(IDENT, "abc")"#);
        assert_eq!(inline_print(&hidden_token), r#"" ""#);
        assert_eq!(
            inline_print(&tree),
            r#"Tree("pair", [Token(IDENT, "abc"), " "])"#
        );
        assert_eq!(tree.get_text(), inline_print(&tree));
    }

    #[test]
    fn pretty_print_handles_token_and_tree_inputs_without_panicking() {
        let token = token_ast("abc", "IDENT");
        let tree = sample_tree();

        pretty_print(&token, String::new());
        pretty_print(&tree, String::new());
        token.print();
        tree.pretty_print();
    }
}
