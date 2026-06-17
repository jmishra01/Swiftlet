use std::collections::VecDeque;
use std::fmt::Display;
use crate::lexer::Token;
use std::sync::Arc;

/// Represents either a token leaf or a named tree node in the parse result.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Ast {
    Token(Arc<Token>),
    Tree(String, Vec<Ast>),
}

impl Ast {
    /// Returns the tree node name if this is an `AST::Tree`, otherwise `None`.
    pub fn tree_name(&self) -> Option<&str> {
        match self {
            Ast::Tree(name, _) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Returns `(name, children)` if this is an `Ast::Tree`, otherwise `None`.
    pub fn as_tree(&self) -> Option<(&str, &[Ast])> {
        match self {
            Ast::Tree(name, children) => Some((name.as_str(), children.as_slice())),
            _ => None,
        }
    }

    /// Returns the inner token if this is an `Ast::Token`, otherwise `None`
    pub fn as_token(&self) -> Option<&Arc<Token>> {
        match self {
            Ast::Token(tok) => Some(tok),
            _ => None,
        }
    }

    /// Return `true` when this node is suppressed from the tree by underscore naming rules
    /// (`_name` but NOT `__name`; double-underscore terminals are raw and always visible).
    pub fn is_suppressed(&self) -> bool {
        match self {
            Ast::Token(token) => token.terminal_is_hidden,
            Ast::Tree(name, _) => name.starts_with('_') && !name.starts_with("__"),
        }
    }

    /// Returns `true` when this is a raw/anonymous terminal (`__name` prefix).
    pub fn is_anonymous(&self) -> bool {
        match self {
            Ast::Token(token) => token.terminal.as_str().starts_with("__"),
            Ast::Tree(name, _) => name.starts_with("__"),
        }
    }

    /// Prints a multi-line pretty representation of the AST.
    pub fn pretty_print(&self) {
        pretty_print(self, 0);
    }

    /// Prints a single-line AST representation.
    pub fn print(&self) {
        println!("{}", self.inline_text());
    }

    /// Return `true` if any descendent tree node has the given name
    pub fn contains_tree(&self, tree_name: &str) -> bool {
        match self {
            Ast::Token(_) => false,
            Ast::Tree(name, children) => {
                if *name == tree_name {
                    return true;
                }
                children.iter().any(|child| child.contains_tree(tree_name))
            }
        }
    }

    /// Returns the children slice if this is an `Ast::Tree`, otherwise `None`.
    pub fn children(&self) -> Option<&[Ast]> {
        match self {
            Ast::Tree(_, children) => Some(children.as_slice()),
            _ => None,
        }
    }

    /// Returns a single-line AST representation as a `String`.
    pub fn inline_text(&self) -> String {
        inline_print(self)
    }

    /// Returns the first subtree with the given name (depth-first).
    pub fn tree<'a>(&'a self, tree_name: &'a str) -> Option<&'a Ast> {
        self.iter_trees(tree_name).next()
    }

    /// Lazily iterates over every subtree with the given name (depth-first, left-to-right).
    pub fn iter_trees<'a>(&'a self, tree_name: &'a str) -> AstTreeIter<'a> {
        AstTreeIter::new(self, tree_name)
    }

    /// Collects all subtrees with the given name into a `Vec`
    pub fn trees_named(&self, tree_name: &str) -> Option<Vec<&Ast>> {
        match self {
            Ast::Token(_) => None,
            Ast::Tree(name, children) => {
                let mut ast_vec = Vec::new();
                if name == tree_name {
                    ast_vec.push(self);
                }
                for child in children {
                    if let Some(rule) = child.trees_named(tree_name) {
                        ast_vec.extend(rule);
                    }
                }
                Some(ast_vec)
            }
        }
    }

    /// Returns the last child of this tree node, or `Node` if it is a token or empty.
    pub fn last_child(&self) -> Option<&Ast> {
        match self {
            Ast::Token(_) => None,
            Ast::Tree(_, children) => children.last(),
        }
    }

    /// Returns a depth-first (pre-order, left-to-right) iterator over **every** node in
    /// this subtree -- both `Tree` and `Token` variants.
    ///
    /// Use this as the building block for custom searches. For name-filtered tree-only
    /// traversal prefer [`iter_trees`](Self::iter_trees)
    pub fn iter_all(&self) -> AstDfsIter<'_> {
        AstDfsIter {stack: vec![self]}
    }

    /// Returns a breadth-first iterators **every** node in this subtree.
    ///
    /// Each level is visited left-to-right before descending, so ancestors always
    /// appear before their descendants.
    pub fn iter_subtree(&self) -> AstBfsIter<'_> {
        let mut queue = VecDeque::new();
        queue.push_back(self);
        AstBfsIter {queue}
    }

    /// Returns an iterator over every leaf [`Token`] in this subtree, depth first.
    ///
    /// ``ìgnore
    /// let words: Vec<_> = ast.tokens().map(|t| t.word()).collect();
    pub fn tokens(&self) -> impl Iterator<Item = &Arc<Token>> + '_ {
        self.iter_all().filter_map(|n| match n {
            Ast::Token(tok) => Some(tok),
            _ => None,
        })
    }

    /// Returns an iterator over every token whose terminal name equals `terminal`,
    /// depth-first.
    ///
    /// ```ignore
    /// let nums: Vec<_> = ast.find_tokens("INT").map(|t| t.word()).collect();
    /// ```
    pub fn find_tokens<'a>(&'a self, terminal: &'a str) -> impl Iterator<Item = &'a Arc<Token>> + 'a {
        self.iter_all().filter_map(move |n| match n {
            Ast::Token(tok) if tok.terminal() == terminal => Some(tok),
            _ => None,
        })
    }

    /// Returns an iterator over every node in this subtree for which `pred` returns `true`.
    ///
    /// `pred` receives the node itself, so it can inspect both `Tree` names and `Token` terminals.
    ///
    /// ```ignore
    /// let exprs: Vec<_> = ast.find_pred(|n| n.tree_name() == Some("expr")).collect();
    /// ```
    pub fn find_pred<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a Ast> + 'a
    where
        F: Fn(&Ast) -> bool + 'a, {
        self.iter_all().filter(move |n| predicate(*n))
    }

    /// Yields the `.word()` text of every token for which `pred` returns `true`, depth-first.
    ///
    /// ```ignore
    /// let values: Vec<_> = ast.scan_values(|t| t.terminal() == "INT").collect();
    /// ```
    pub fn scan_values<'a, F>(&'a self, pred: F) -> impl Iterator<Item = &'a str> + 'a
    where
        F: Fn(&Token) -> bool + 'a, {
        self.iter_all().filter_map(move |n| match n {
            Ast::Token(tok) if pred(tok.as_ref()) => Some(tok.word()),
            _ => None,
        })
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inline_text())
    }
}

/// Depth-first iterator over matching AST tree nodes.
pub struct AstTreeIter<'a> {
    stack: Vec<&'a Ast>,
    tree_name: &'a str,
}

impl<'a> AstTreeIter<'a> {
    fn new(root: &'a Ast, tree_name: &'a str) -> Self {
        AstTreeIter {
            stack: vec![root],
            tree_name,
        }
    }
}

impl<'a> Iterator for AstTreeIter<'a> {
    type Item = &'a Ast;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if let Ast::Tree(name, children) = node {
                self.stack.extend(children.iter().rev());
                if name == self.tree_name {
                    return Some(node);
                }
            }
        }
        None
    }
}

/// Depth-first (pre-order) iterator over **all**AST nodes
///
/// Return by [`Ast::iter_all`]. Visits each node before its children, left-to-right.
pub struct AstDfsIter<'a> {
    stack: Vec<&'a Ast>,
}

impl<'a> Iterator for AstDfsIter<'a> {
    type Item = &'a Ast;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        if let Ast::Tree(_, children) = node {
            // Push in reverse so leftmost child is popped first.
            self.stack.extend(children.iter().rev());
        }
        Some(node)
    }
}

/// Breadth-first iterator over **all** AST nodes.
///
/// Returned by [`Ast::iter_subtree`]. Visits each level left-to-right before descending.
pub struct AstBfsIter<'a> {
    queue: VecDeque<&'a Ast>,
}

impl<'a> Iterator for AstBfsIter<'a> {
    type Item = &'a Ast;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        if let Ast::Tree(_, children) = node {
            self.queue.extend(children.iter());
        }
        Some(node)
    }
}


/// Converts an AST to a compact single-line textual form.
fn inline_print(tree: &Ast) -> String {
    let mut buf = String::new();
    write_inline(tree, &mut buf);
    buf
}

/// Writes the inline representation of `tree` into `buf`, avoiding intermediate allocations.
fn write_inline(tree: &Ast, buf: &mut String) {
    match tree {
        Ast::Token(token) => {
            let terminal = token.terminal.as_str();
            if terminal.starts_with("__") {
                buf.push_str(&format!("{:?}", token.word()));
            } else {
                buf.push_str("Token(");
                buf.push_str(terminal);
                buf.push_str(", \"");
                buf.push_str(token.word());
                buf.push_str("\")");
            }
        },
        Ast::Tree(name, children) => {
            buf.push_str("Tree(\"");
            buf.push_str(name);
            buf.push_str("\", [");
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                write_inline(child, buf);
            }
            buf.push_str("])");
        }
    }
}

/// Recursively pretty-prints an AST with indentation.
fn pretty_print(tree: &Ast, depth: usize) {
    let space = " ".repeat(depth);
    match tree {
        Ast::Token(name) => println!("{}{}", space, name.word()),
        Ast::Tree(name, v_ast) => {
            if v_ast.len() == 1 && matches!(v_ast[0], Ast::Token(_)) {
                let val = match &v_ast[0] {
                    Ast::Token(token) => token.word(),
                    _ => panic!("Not a token"),
                };
                println!("{}{}  {}", space, name, val);
                return;
            }

            println!("{}{}", space, name);
            for _ast in v_ast {
                pretty_print(_ast, depth + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Symbol;

    fn token_ast(word: &str, terminal: &str) -> Ast {
        Ast::Token(Arc::new(Token::new(
            Arc::<str>::from(word),
            0,
            word.len(),
            0,
            Arc::new(Symbol::Terminal(terminal.to_string())),
        )))
    }

    fn sample_tree() -> Ast {
        Ast::Tree(
            "root".to_string(),
            vec![
                Ast::Tree("expr".to_string(), vec![token_ast("a", "IDENT")]),
                Ast::Tree(
                    "_hidden".to_string(),
                    vec![
                        Ast::Tree("expr".to_string(), vec![token_ast("b", "IDENT")]),
                        Ast::Tree("leaf".to_string(), vec![token_ast("c", "__RAW")]),
                    ],
                ),
            ],
        )
    }

    #[test]
    fn ast_struct_and_basic_helpers_work() {
        let ast_tok = token_ast("hello", "_WS");
        let ast_tree = Ast::Tree("node".to_string(), vec![ast_tok.clone()]);

        assert_eq!(ast_tree.tree_name(), Some("node"));
        assert!(ast_tok.is_suppressed());
        assert!(!ast_tree.is_suppressed());
        assert_eq!(ast_tree.children().map(<[_]>::len), Some(1));
        assert_eq!(ast_tok.children(), None);
        assert_eq!(ast_tree.last_child(), Some(&ast_tok));
        assert_eq!(ast_tok.last_child(), None);
        assert!(ast_tree.inline_text().starts_with("Tree(\"node\""));
    }

    #[test]
    fn as_tree_and_as_token_work() {
        let tok = token_ast("x", "ID");
        let tree = Ast::Tree("expr".to_string(), vec![tok.clone()]);

        let (name, children) = tree.as_tree().unwrap();
        assert_eq!(name, "expr");
        assert_eq!(children.len(), 1);
        assert!(tree.as_token().is_none());
        assert!(tok.as_token().is_some());
        assert!(tok.as_tree().is_none());
    }

    #[test]
    fn is_anonymous_detects_double_underscore() {
        let raw = token_ast("x", "__RAW");
        let hidden = token_ast("x", "_WS");
        let normal = token_ast("x", "IDENT");

        assert!(raw.is_anonymous());
        assert!(!hidden.is_anonymous());
        assert!(!normal.is_anonymous());
    }

    #[test]
    fn display_delegates_to_inline_text() {
        let tree = Ast::Tree("start".to_string(), vec![token_ast("1", "INT")]);
        assert_eq!(format!("{tree}"), tree.inline_text())
    }

    #[test]
    fn tree_search_helpers_find_expected_nodes() {
        let tree = sample_tree();

        assert!(tree.contains_tree("expr"));
        assert!(tree.contains_tree("leaf"));
        assert!(!tree.contains_tree("missing"));

        let first_expr = tree.tree("expr");
        assert!(matches!(first_expr, Some(Ast::Tree(name, _)) if name == "expr"));

        let expr_nodes = tree
            .trees_named("expr")
            .expect("expr nodes should exist");
        assert_eq!(expr_nodes.len(), 2);

        let leaf_nodes = tree
            .trees_named("leaf")
            .expect("leaf nodes should exist");
        assert_eq!(leaf_nodes.len(), 1);

        assert!(matches!(tree.tree("missing"), None));
        assert_eq!(token_ast("x", "IDENT").trees_named("expr"), None);
    }

    #[test]
    fn iter_trees_yields_matches_in_depth_first_left_to_right_order() {
        let tree = sample_tree();

        let names = tree
            .iter_trees("expr")
            .map(|ast| ast.tree_name().expect("tree nodes only").to_string())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["expr".to_string(), "expr".to_string()]);
    }

    #[test]
    fn inline_print_formats_tokens_and_trees() {
        let regular_token = token_ast("abc", "IDENT");
        let hidden_token = token_ast(" ", "__WS");
        let tree = Ast::Tree(
            "pair".to_string(),
            vec![regular_token.clone(), hidden_token.clone()],
        );

        assert_eq!(inline_print(&regular_token), r#"Token(IDENT, "abc")"#);
        assert_eq!(inline_print(&hidden_token), r#"" ""#);
        assert_eq!(
            inline_print(&tree),
            r#"Tree("pair", [Token(IDENT, "abc"), " "])"#
        );
        assert_eq!(tree.inline_text(), inline_print(&tree));
    }

    #[test]
    fn pretty_print_handles_token_and_tree_inputs_without_panicking() {
        let token = token_ast("abc", "IDENT");
        let tree = sample_tree();

        pretty_print(&token, 0);
        pretty_print(&tree, 0);
        token.print();
        tree.pretty_print();
    }

    #[test]
    fn tree_name_returns_none_for_token_nodes() {
        let tok = token_ast("hello", "ID");
        assert_eq!(tok.tree_name(), None);
    }

    #[test]
    fn is_anonymous_detects_double_underscore_prefix_for_tree_nodes() {
        let anon = Ast::Tree("__raw".to_string(), vec![]);
        let hidden = Ast::Tree("_hidden".to_string(), vec![]);
        let normal = Ast::Tree("expr".to_string(), vec![]);
        assert!(anon.is_anonymous());
        assert!(!hidden.is_anonymous());
        assert!(!normal.is_anonymous());
    }

    // ---- New traversal helpers --------

    /// Maps each node to a short label used in ordering assertions.
    fn node_label(n: &Ast) -> &str {
        match n {
            Ast::Tree(name, _) => name.as_str(),
            Ast::Token(tok) => tok.terminal()
        }
    }

    #[test]
    fn iter_all_visits_every_node_depth_first_preorder() {
        // Tree layout:
        //  root
        //  | --- expr -> Token(IDENT, "a")
        //  |
        //  `--- _hidden
        //          |
        //          |--- Expr -> Token(IDENT, "b")
        //          |--- leaf -> Token(__RAW, "c")
        let tree = sample_tree();
        let labels: Vec<_> = tree.iter_all().map(node_label).collect();
        assert_eq!(labels, vec!["root", "expr", "IDENT", "_hidden", "expr", "IDENT", "leaf", "__RAW"]);
    }

    #[test]
    fn iter_all_on_token_leaf_yields_single_node() {
        let tok = token_ast("x", "ID");
        let labels: Vec<_> = tok.iter_all().map(node_label).collect();
        assert_eq!(labels, vec!["ID"])
    }

    #[test]
    fn iter_subtree_visits_every_node_breadth_first() {
        let tree = sample_tree();
        let labels: Vec<_> = tree.iter_subtree().map(node_label).collect();
        // Level 0: root
        // Level 1: expr, _hidden
        // Level 2: IDENT("a"), expr, leaf
        // Level 3: IDENT("b"), __RAW("c")
        assert_eq!(labels, vec!["root", "expr", "_hidden", "IDENT", "expr", "leaf", "IDENT", "__RAW"]);
    }

    #[test]
    fn iter_subtree_on_token_leaf_yields_single_node() {
        let tok = token_ast("y", "NUM");
        let labels: Vec<_> = tok.iter_subtree().map(node_label).collect();
        assert_eq!(labels, vec!["NUM"])
    }

    #[test]
    fn tokens_yields_all_leaf_tokens_depth_first() {
        let tree = sample_tree();
        let words: Vec<_> = tree.tokens().map(|t| t.word()).collect();
        assert_eq!(words, vec!["a", "b", "c"]);
    }

    #[test]
    fn tokens_on_token_node_yields_itself() {
        let tok = token_ast("hello", "IDENT");
        let words: Vec<_> = tok.tokens().map(|t| t.word()).collect();
        assert_eq!(words, vec!["hello"]);
    }

    #[test]
    fn find_tokens_filters_by_terminal_name() {
        let tree  = sample_tree();

        // Only IDENT tokens
        let ident_words: Vec<_> = tree.find_tokens("IDENT").map(|t| t.word()).collect();
        assert_eq!(ident_words, vec!["a","b"]);

        // Only __RAW tokens
        let raw_words: Vec<_> = tree.find_tokens("__RAW").map(|t| t.word()).collect();
        assert_eq!(raw_words, vec!["c"]);

        // Terminal that doesn't exist
        assert_eq!(tree.find_tokens("INT").count(), 0);
    }

    #[test]
    fn find_pred_filters_nodes_by_arbitrary_predicate() {
        let tree = sample_tree();

        // All Tree nodes name "expr"
        let expr_nodes: Vec<_> = tree.find_pred(|n| n.tree_name() == Some("expr")).collect();
        assert_eq!(expr_nodes.len(), 2);
        assert!(expr_nodes.iter().all(|n| n.tree_name() == Some("expr")));

        // All Token nodes
        let token_nodes: Vec<_> = tree.find_pred(|n| n.as_token().is_some()).collect();
        assert_eq!(token_nodes.len(), 3);

        // Predicate that matches nothing
        assert_eq!(tree.find_pred(|n| n.tree_name() == Some("missing")).count(), 0);
    }

    #[test]
    fn find_pred_on_token_leaf_matches_or_not() {
        let tok = token_ast("42", "INT");

        let matched: Vec<_> = tok.find_pred(|n| n.as_token().is_some()).collect();
        assert_eq!(matched.len(), 1);

        let no_matched: Vec<_> = tok.find_pred(|n| n.tree_name().is_none()).collect();
        assert!(!no_matched.is_empty());
    }

    #[test]
    fn scan_values_yields_words_of_matching_tokens() {
        let tree = sample_tree();

        // Words of all IDENT tokens.
        let ident_vals: Vec<_> = tree.scan_values(|t| t.terminal() == "IDENT").collect();
        assert_eq!(ident_vals, vec!["a", "b"]);

        // Words of all tokens (no filter)
        let all_vals: Vec<_> = tree.scan_values(|_| true).collect();
        assert_eq!(all_vals, vec!["a", "b", "c"]);

        // Filter that matches nothing
        assert_eq!(tree.scan_values(|t| t.terminal() == "NUM").count(), 0);
    }

    #[test]
    fn scan_values_on_token_leaf_yields_word_when_pred_matches() {
        let tok = token_ast("99", "INT");

        let vals: Vec<_> = tok.scan_values(|t| t.terminal() == "INT").collect();
        assert_eq!(vals, vec!["99"]);

        let empty: Vec<_> = tok.scan_values(|t| t.terminal() == "FLOAT").collect();
        assert!(empty.is_empty());
    }
}