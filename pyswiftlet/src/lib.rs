use pyo3::exceptions::{PyIndexError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};
use swiftlet::{
    Ambiguity as RustAmbiguity, Parser as RustParser, ParserConfig,
    Swiftlet as RustSwiftlet, ast::Ast, grammar::Algorithm as RustAlgorithm,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_algorithm(value: &str) -> PyResult<RustAlgorithm> {
    match value.to_ascii_lowercase().as_str() {
        "earley" => Ok(RustAlgorithm::Earley),
        "clr" => Ok(RustAlgorithm::CLR),
        _ => Err(PyValueError::new_err(format!(
            "invalid algorithm '{value}', expected 'earley' or 'clr'"
        ))),
    }
}

fn parse_ambiguity(value: &str) -> PyResult<RustAmbiguity> {
    match value.to_ascii_lowercase().as_str() {
        "resolve" => Ok(RustAmbiguity::Resolve),
        "explicit" => Ok(RustAmbiguity::Explicit),
        _ => Err(PyValueError::new_err(format!(
            "invalid ambiguity '{value}', expected 'resolve' or 'explicit'"
        ))),
    }
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    if let Some(s) = payload.downcast_ref::<&str>() {
        return (*s).to_string();
    }
    "swiftlet panicked".to_string()
}

fn build_parser_config(
    start: &str,
    algorithm: &str,
    ambiguity: &str,
    debug: bool,
) -> PyResult<Arc<ParserConfig>> {
    Ok(Arc::new(ParserConfig {
        start: start.to_string(),
        algorithm: parse_algorithm(algorithm)?,
        ambiguity: parse_ambiguity(ambiguity)?,
        debug,
    }))
}

/// Generic constructor — avoids naming swiftlet's private `SwiftletError` type.
fn build_parser<E: std::fmt::Display>(
    make: impl FnOnce(Arc<ParserConfig>) -> Result<RustParser, E> + std::panic::UnwindSafe,
    start: &str,
    algorithm: &str,
    ambiguity: &str,
    debug: bool,
) -> PyResult<RustParser> {
    let config = build_parser_config(start, algorithm, ambiguity, debug)?;
    catch_unwind(|| make(config))
        .map_err(|p| PyRuntimeError::new_err(panic_payload_to_string(p)))?
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// A matched terminal token with its source span.
///
/// Attributes:
///     word (str):      Matched text.
///     start (int):     Start byte offset in the source.
///     end (int):       End byte offset in the source.
///     line (int):      Zero-based source line.
///     terminal (str):  Terminal name from the grammar.
#[pyclass(frozen, module = "swiftlet._core", skip_from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub word: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub terminal: String,
}

#[pymethods]
impl Token {
    #[new]
    #[pyo3(signature = (word, start, end, line, terminal))]
    fn new(word: String, start: usize, end: usize, line: usize, terminal: String) -> Self {
        Self { word, start, end, line, terminal }
    }

    fn get_word(&self) -> &str {
        &self.word
    }

    fn get_terminal(&self) -> &str {
        &self.terminal
    }

    fn get_start(&self) -> usize {
        self.start
    }

    fn get_end(&self) -> usize {
        self.end
    }

    fn get_line(&self) -> usize {
        self.line
    }

    fn __len__(&self) -> usize {
        self.word.len()
    }

    fn __str__(&self) -> String {
        if self.terminal.starts_with("__") {
            self.word.clone()
        } else {
            format!("Token({}, \"{}\")", self.terminal, self.word)
        }
    }

    fn __repr__(&self) -> String {
        if self.terminal.starts_with("__") {
            self.word.clone()
        } else {
            format!(
                "Token(Type: {}, Word: \"{}\", Start: {}, End: {}, Line: {})",
                self.terminal, self.word, self.start, self.end, self.line
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

/// A parse-tree node produced by a grammar rule.
///
/// Attributes:
///     name (str):      Grammar rule name.
///     children (list): Child ``Tree`` / ``Token`` nodes.
#[pyclass(module = "swiftlet._core")]
pub struct Tree {
    name: String,
    children: Vec<Py<PyAny>>,
}

#[pymethods]
impl Tree {
    #[new]
    #[pyo3(signature = (name, children))]
    pub fn new(name: String, children: Vec<Py<PyAny>>) -> Self {
        Self { name, children }
    }

    fn get_children(&self) -> &Vec<Py<PyAny>> {
        &self.children
    }

    fn set_children(&mut self, children: Vec<Py<PyAny>>) {
        self.children = children;
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns `True` if this node is suppressed (``_name`` but not ``__name``).
    fn is_suppressed(&self) -> bool {
        self.name.starts_with('_') && !self.name.starts_with("__")
    }

    /// Returns `True` if this is a raw/anonymous terminal (``__name`` prefix).
    fn is_anonymous(&self) -> bool {
        self.name.starts_with("__")
    }

    /// Returns `True` if any descendant tree node has the given name.
    fn contains_tree(&self, py: Python<'_>, tree_name: &str) -> bool {
        if self.name == tree_name {
            return true;
        }
        self.children.iter().any(|child| {
            if let Ok(subtree) = child.bind(py).cast::<Tree>() {
                subtree.borrow().contains_tree(py, tree_name)
            } else {
                false
            }
        })
    }

    /// Returns the first descendant tree with the given name (depth-first).
    fn find_tree(&self, py: Python<'_>, tree_name: &str) -> PyResult<Option<Py<PyAny>>> {
        if self.name == tree_name {
            let children = self.children.iter().map(|c| c.clone_ref(py)).collect();
            return Ok(Some(
                Py::new(py, Tree { name: self.name.clone(), children })?.into_any(),
            ));
        }
        for child in &self.children {
            if let Ok(subtree) = child.bind(py).cast::<Tree>() {
                if let Some(found) = subtree.borrow().find_tree(py, tree_name)? {
                    return Ok(Some(found));
                }
            }
        }
        Ok(None)
    }

    /// Returns all descendant trees with the given name (depth-first).
    fn find_all_trees(&self, py: Python<'_>, tree_name: &str) -> PyResult<Vec<Py<PyAny>>> {
        let mut results: Vec<Py<PyAny>> = Vec::new();
        if self.name == tree_name {
            let children = self.children.iter().map(|c| c.clone_ref(py)).collect();
            results.push(Py::new(py, Tree { name: self.name.clone(), children })?.into_any());
        }
        for child in &self.children {
            if let Ok(subtree) = child.bind(py).cast::<Tree>() {
                results.extend(subtree.borrow().find_all_trees(py, tree_name)?);
            }
        }
        Ok(results)
    }

    /// Returns the last child node, or `None` if the tree is empty.
    fn last_child(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.children.last().map(|c| c.clone_ref(py))
    }

    /// Returns the start byte offset of the first descendant token.
    fn get_start(&self, py: Python<'_>) -> PyResult<usize> {
        let first = self
            .children
            .first()
            .ok_or_else(|| PyRuntimeError::new_err("Tree has no children"))?
            .bind(py);
        if let Ok(subtree) = first.cast::<Tree>() {
            subtree.borrow().get_start(py)
        } else if let Ok(tok) = first.cast::<Token>() {
            // `Token` is frozen → `.get()` skips the GIL-based borrow lock.
            Ok(tok.get().get_start())
        } else {
            Err(PyRuntimeError::new_err("expected Tree or Token child"))
        }
    }

    /// Returns the end byte offset of the last descendant token.
    fn get_end(&self, py: Python<'_>) -> PyResult<usize> {
        let last = self
            .children
            .last()
            .ok_or_else(|| PyRuntimeError::new_err("Tree has no children"))?
            .bind(py);
        if let Ok(subtree) = last.cast::<Tree>() {
            subtree.borrow().get_end(py)
        } else if let Ok(tok) = last.cast::<Token>() {
            Ok(tok.get().get_end())
        } else {
            Err(PyRuntimeError::new_err("expected Tree or Token child"))
        }
    }

    fn __len__(&self) -> usize {
        self.children.len()
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let refs: Vec<Py<PyAny>> = slf.children.iter().map(|c| c.clone_ref(py)).collect();
        Ok(PyList::new(py, refs)?.call_method0("__iter__")?.unbind())
    }

    fn __getitem__(&self, py: Python<'_>, index: isize) -> PyResult<Py<PyAny>> {
        let len = self.children.len() as isize;
        let idx = if index < 0 { len + index } else { index };
        if idx < 0 || idx >= len {
            return Err(PyIndexError::new_err("index out of range"));
        }
        Ok(self.children[idx as usize].clone_ref(py))
    }

    fn __str__(&self) -> String {
        let child = self.children.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ");
        format!("Tree({}, [{}])", self.name, child)
    }

    fn __repr__(&self) -> PyResult<String> {
        let child = Python::attach(|py| {
            self.children
                .iter()
                .map(|c| {
                    c.call_method0(py, "__repr__")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|_| "?".to_string())
                })
                .collect::<Vec<_>>()
                .join(", ")
        });
        Ok(format!("Tree({}, [{}])", self.name, child))
    }
}

// ---------------------------------------------------------------------------
// AST conversion
// ---------------------------------------------------------------------------

fn convert_to_py(py: Python<'_>, ast: &Ast) -> PyResult<Py<PyAny>> {
    match ast {
        Ast::Token(token) => {
            let py_token = Token {
                word: token.word().to_string(),
                start: token.start(),
                end: token.end(),
                line: token.line(),
                terminal: token.terminal.get_value(),
            };
            Ok(py_token.into_pyobject(py)?.into_any().unbind())
        }
        Ast::Tree(name, children) => {
            let child = children
                .iter()
                .map(|c| convert_to_py(py, c))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(Tree { name: name.to_string(), children: child }
                .into_pyobject(py)?
                .into_any()
                .unbind())
        }
    }
}

// ---------------------------------------------------------------------------
// Swiftlet
// ---------------------------------------------------------------------------

/// Error variants returned across the `allow_threads` boundary (no Python objects).
enum ParseErr {
    Lock,
    Panic(String),
    Parse(String),
}

/// Compiled grammar and parser.  Thread-safe; the GIL is released while parsing.
///
/// Args:
///     grammar (str):    EBNF grammar definition.
///     start (str):      Start rule name (default: ``"start"``).
///     algorithm (str):  ``"earley"`` (any CFG, default) or ``"clr"``
///                       (LR(1) — faster for unambiguous grammars).
///     ambiguity (str):  ``"resolve"`` (default) or ``"explicit"``.
///     debug (bool):     Print internal parse tables (default: ``False``).
#[pyclass(frozen, module = "swiftlet._core")]
pub struct Swiftlet {
    inner: Arc<Mutex<RustParser>>,
}

#[pymethods]
impl Swiftlet {
    #[new]
    #[pyo3(signature = (grammar, start="start", algorithm="earley", ambiguity="resolve", debug=false))]
    fn new(grammar: &str, start: &str, algorithm: &str, ambiguity: &str, debug: bool) -> PyResult<Self> {
        let grammar = grammar.to_string();
        let parser = build_parser(
            move |cfg| RustSwiftlet::from_str(&grammar).map(|g| g.parser((*cfg).clone())),
            start, algorithm, ambiguity, debug,
        )?;
        Ok(Self { inner: Arc::new(Mutex::new(parser)) })
    }

    #[staticmethod]
    #[pyo3(signature = (file, start="start", algorithm="earley", ambiguity="resolve", debug=false))]
    /// Constructs a parser from a grammar file path.
    fn from_file(file: &str, start: &str, algorithm: &str, ambiguity: &str, debug: bool) -> PyResult<Self> {
        let file = file.to_string();
        let parser = build_parser(
            move |cfg| RustSwiftlet::from_file(&file).map(|g| g.parser((*cfg).clone())),
            start, algorithm, ambiguity, debug,
        )?;
        Ok(Self { inner: Arc::new(Mutex::new(parser)) })
    }

    /// Parses *text* and returns a ``Tree`` or ``Token``.
    ///
    /// The GIL is released during the Rust parse so other Python threads
    /// can run concurrently.
    fn parse(&self, py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
        let inner = Arc::clone(&self.inner);
        let text = text.to_string();

        let result = py.detach(move || -> Result<Ast, ParseErr> {
            let parser = inner.lock().map_err(|_| ParseErr::Lock)?;
            catch_unwind(AssertUnwindSafe(|| parser.parse(&text)))
                .map_err(|p| ParseErr::Panic(panic_payload_to_string(p)))?
                .map_err(|e| ParseErr::Parse(e.to_string()))
        });

        let ast = result.map_err(|e| match e {
            ParseErr::Lock => PyRuntimeError::new_err("failed to acquire parser lock"),
            ParseErr::Panic(s) => PyRuntimeError::new_err(s),
            ParseErr::Parse(s) => PyValueError::new_err(s),
        })?;

        convert_to_py(py, &ast)
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[pymodule]
#[pyo3(name = "_swiftlet")]
fn core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Swiftlet>()?;
    module.add_class::<Tree>()?;
    module.add_class::<Token>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
