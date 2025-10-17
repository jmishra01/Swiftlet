// use regex::Regex;
use fancy_regex::{Regex, RegexBuilder};
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AST {
    Token(Arc<Token>),
    Tree(String, Vec<AST>),
}

impl AST {
    pub fn get_tree_name(&self) -> Option<&String> {
        match self {
            AST::Tree(name, _) => Some(name),
            _ => None,
        }
    }

    pub fn is_start_with_underscore(&self) -> bool {
        match self {
            AST::Token(token) => {
                token.terminal.get_value().starts_with("_")
                    && !token.terminal.get_value().starts_with("__")
            }
            AST::Tree(name, _) => name.starts_with("_") && !name.starts_with("__"),
        }
    }

    pub fn pretty_print(&self) {
        pretty_print(self, "".to_string());
    }

    pub fn print(&self) {
        println!("{}", self.get_text());
    }

    pub fn get_text(&self) -> String {
        inline_print(self)
    }
}

fn inline_print(tree: &AST) -> String {
    match tree {
        AST::Token(token) => token.word.clone(),
        AST::Tree(name, children) => {
            let c = children
                .iter()
                .map(inline_print)
                .collect::<Vec<String>>()
                .join(", ");
            format!("{}([{}])", name, c)
        }
    }
}

fn pretty_print(tree: &AST, space: String) {
    match tree {
        AST::Token(name) => println!("{}{:?}", space, name.word),
        AST::Tree(name, v_ast) => {
            println!("{}{}", space, name);
            let _rep = " ".to_string().repeat(name.len().div_ceil(2));
            let rep = _rep.as_str();
            for _ast in v_ast {
                pretty_print(_ast, space.clone() + rep);
            }
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Symbol {
    Terminal(String),
    NonTerminal(String),
}

impl Symbol {
    pub fn get_value(&self) -> String {
        match self {
            Symbol::Terminal(value) => value.clone(),
            Symbol::NonTerminal(value) => value.clone(),
        }
    }

    #[inline(always)]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Symbol::Terminal(_))
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        match self {
            Symbol::Terminal(value) => value.starts_with(prefix),
            Symbol::NonTerminal(value) => value.starts_with(prefix),
        }
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Symbol::NonTerminal(name) => format!("NonTerminal({name})"),
                Symbol::Terminal(name) => format!("Terminal({name})"),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    PatternStr(String),
    PatternRegex(Regex),
}

impl Pattern {
    pub(crate) fn capture(&self, text: &str) -> Option<(String, usize, usize)> {
        match self {
            Pattern::PatternStr(name) => {
                if text.starts_with(name) {
                    return Some((name.to_string(), text.len(), name.len()));
                }
                None
            }
            Pattern::PatternRegex(regex) => {
                if let Ok(Some(caps)) = regex.captures(text)
                    && caps.len() > 0
                    && let Some(mt) = caps.get(0)
                {
                    return Some((mt.as_str().to_string(), mt.end(), mt.as_str().len()));
                }
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerminalDef {
    pub(crate) name: Arc<Symbol>,
    #[allow(dead_code)]
    pub(crate) value: String,
    pub(crate) pattern: Pattern,
    pub(crate) max_width: usize,
}

impl TerminalDef {
    pub fn new(name: &str, value: &str, regex: bool, case_insensitive: bool) -> Self {
        let name = Arc::new(Symbol::Terminal(name.to_string()));

        let (pattern, max_width) = {
            if regex {
                let rb = RegexBuilder::new((r"^".to_string() + value).as_str())
                    .case_insensitive(case_insensitive)
                    .build()
                    .unwrap();
                let pr = Pattern::PatternRegex(rb);
                let max = if value.contains("+") || value.contains("*") {
                    usize::MAX
                } else {
                    value.len()
                };
                (pr, max)
            } else {
                (Pattern::PatternStr(value.to_string()), value.len())
            }
        };

        Self {
            name,
            value: value.to_string(),
            pattern,
            max_width,
        }
    }

    pub fn get_name(&self) -> Arc<Symbol> {
        self.name.clone()
    }

    fn capture(&self, text: &str) -> Option<(String, usize, usize)> {
        self.pattern.capture(text)
    }
}

impl PartialEq for TerminalDef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

#[cfg(feature = "debug")]
impl Display for TerminalDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {:?}", self.name.get_value(), self.pattern)
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub word: String,
    start: usize,
    end: usize,
    line: usize,
    pub terminal: Arc<Symbol>,
}

impl Token {
    pub fn new(word: String, start: usize, end: usize, line: usize, terminal: Arc<Symbol>) -> Self {
        Self {
            word,
            start,
            end,
            line,
            terminal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer {
    text: String,
    start: usize,
    line: usize,
    len: usize,
    terminals: Vec<Arc<TerminalDef>>,
    ignore: Vec<String>,
}

impl Tokenizer {
    pub(crate) fn new(text: &str, terminals: &[Arc<TerminalDef>], ignore: &[String]) -> Self {
        Self {
            text: text.to_string(),
            start: 0usize,
            line: 0usize,
            len: text.len(),
            terminals: terminals.to_owned(),
            ignore: ignore.to_vec(),
        }
    }

    pub(crate) fn get_text(&self) -> &str {
        &self.text
    }
}

impl Iterator for Tokenizer {
    type Item = Arc<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.len {
            let slice_text = &self.text[self.start..];

            let previous_start = self.start;

            for terminal in self.terminals.iter() {
                if let Some((mt_word, mt_end, mt_len)) = terminal.capture(slice_text) {
                    return if !self.ignore.contains(&terminal.name.get_value()) {
                        let token = Arc::new(Token::new(
                            mt_word,
                            self.start,
                            mt_end + self.start,
                            self.line,
                            terminal.name.clone(),
                        ));

                        if terminal.name.get_value() == "_NL" {
                            self.line += 1;
                            if self.start == 0 {
                                self.start += mt_end;
                                return self.next();
                            }
                        }
                        self.start += mt_len;

                        Some(token)
                    } else {
                        self.start += mt_end;
                        self.next()
                    };
                }
            }

            if previous_start == self.start {
                let expected_next_token = self
                    .terminals
                    .iter()
                    .map(|x| match x.pattern.clone() {
                        Pattern::PatternRegex(_) => x.get_name().get_value(),
                        Pattern::PatternStr(name) => name,
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                panic!(
                    "Failed during tokenization at location {} of input text: \"{}\", expecting one of the following terminals: ({}).",
                    previous_start, self.text, expected_next_token
                );
            }
        }
        None
    }
}

#[derive(Debug)]
pub(crate) struct LexerConf {
    pub terminals: Vec<Arc<TerminalDef>>,
}

impl LexerConf {
    pub fn new(terminals: Vec<Arc<TerminalDef>>) -> Self {
        Self { terminals }
    }

    pub fn tokenize(&self, text: &str, ignore: &[String]) -> Tokenizer {
        Tokenizer::new(text, &self.terminals, ignore)
    }
}

pub fn get_symbol(word: &str) -> Arc<Symbol> {
    if word.chars().any(|x| x.is_ascii_lowercase()) {
        return Arc::new(Symbol::NonTerminal(word.to_string()));
    }
    Arc::new(Symbol::Terminal(word.to_string()))
}
