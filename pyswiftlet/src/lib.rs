use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};
use swiftlet::{
    Ambiguity as RustAmbiguity, LexerMode as RustLexerMode, ParserOption, Swiftlet as RustSwiftlet,
    ast::AST, grammar::Algorithm as RustAlgorithm,
};

/// Parses the Python-facing algorithm name into the Rust enum.
fn parse_algorithm(value: &str) -> PyResult<RustAlgorithm> {
    match value.to_ascii_lowercase().as_str() {
        "earley" => Ok(RustAlgorithm::Earley),
        "clr" => Ok(RustAlgorithm::CLR),
        _ => Err(PyValueError::new_err(format!(
            "invalid algorithm '{value}', expected 'earley', 'clr', or 'lalr'"
        ))),
    }
}

/// Parses the Python-facing ambiguity mode into the Rust enum.
fn parse_ambiguity(value: &str) -> PyResult<RustAmbiguity> {
    match value.to_ascii_lowercase().as_str() {
        "resolve" => Ok(RustAmbiguity::Resolve),
        "explicit" => Ok(RustAmbiguity::Explicit),
        _ => Err(PyValueError::new_err(format!(
            "invalid ambiguity '{value}', expected 'resolve' or 'explicit'"
        ))),
    }
}

/// Parses the Python-facing lexer mode into the Rust enum.
fn parse_lexer_mode(value: &str) -> PyResult<RustLexerMode> {
    match value.to_ascii_lowercase().as_str() {
        "basic" => Ok(RustLexerMode::Basic),
        "dynamic" => Ok(RustLexerMode::Dynamic),
        "scannerless" => Ok(RustLexerMode::Scannerless),
        _ => Err(PyValueError::new_err(format!(
            "invalid lexer_mode '{value}', expected 'basic', 'dynamic' or 'scannerless'"
        ))),
    }
}

/// Converts a panic payload into a readable Python error message.
fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    "swiftlet panicked".to_string()
}

/// Builds shared parser options from Python constructor arguments.
fn build_parser_option(
    start: &str,
    algorithm: &str,
    ambiguity: &str,
    lexer_mode: &str,
    debug: bool,
) -> PyResult<Arc<ParserOption>> {
    Ok(Arc::new(ParserOption {
        start: start.to_string(),
        algorithm: parse_algorithm(algorithm)?,
        ambiguity: parse_ambiguity(ambiguity)?,
        lexer_mode: parse_lexer_mode(lexer_mode)?,
        debug,
    }))
}

/// Builds a Rust parser from inline grammar text and maps panics to Python errors.
fn build_parser_from_grammar(
    grammar: &str,
    start: &str,
    algorithm: &str,
    ambiguity: &str,
    lexer_mode: &str,
    debug: bool,
) -> PyResult<RustSwiftlet> {
    let parser_option = build_parser_option(start, algorithm, ambiguity, lexer_mode, debug)?;
    let parser = catch_unwind(AssertUnwindSafe(|| {
        RustSwiftlet::from_string(grammar, parser_option)
    }))
    .map_err(|payload| PyRuntimeError::new_err(panic_payload_to_string(payload)))?;
    parser.map_err(|err| PyValueError::new_err(err.to_string()))
}

/// Builds a Rust parser from a grammar file and maps panics to Python errors.
fn build_parser_from_file(
    file: &str,
    start: &str,
    algorithm: &str,
    ambiguity: &str,
    lexer_mode: &str,
    debug: bool,
) -> PyResult<RustSwiftlet> {
    let parser_option = build_parser_option(start, algorithm, ambiguity, lexer_mode, debug)?;
    let parser = catch_unwind(AssertUnwindSafe(|| {
        RustSwiftlet::from_file(file.to_string(), parser_option)
    }))
    .map_err(|payload| PyRuntimeError::new_err(panic_payload_to_string(payload)))?;
    parser.map_err(|err| PyValueError::new_err(err.to_string()))
}

/// Token
/// =====
/// Signature:
///     Token(word: String, start: Integer, end: Integer, line: Integer, terminal: String)
///
/// Arguments:
///     word:           String  | Value store by token.
///     start:          Integer | Value start in the input string.
///     end:            Integer | Value end in the input string.
///     line:           Integer | Value line in the input string.
///     terminal:       String  | Type of token.
#[pyclass(module = "swiftlet._core", skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct Token {
    word: String,
    start: usize,
    end: usize,
    line: usize,
    terminal: String,
}

#[pymethods]
impl Token {
    #[new]
    #[pyo3(signature = (word, start, end, line, terminal))]
    /// Creates a Python token wrapper.
    fn new(word: String, start: usize, end: usize, line: usize, terminal: String) -> Self {
        Self {
            word,
            start,
            end,
            line,
            terminal,
        }
    }

    /// Returns the matched token text.
    fn get_word(&self) -> &str {
        &self.word
    }

    /// Returns the token terminal name.
    fn get_terminal(&self) -> &str {
        &self.terminal
    }

    /// Returns the token start byte offset.
    fn get_start(&self) -> usize {
        self.start
    }
    /// Returns the token end byte offset.
    fn get_end(&self) -> usize {
        self.end
    }
    /// Returns the zero-based source line for the token.
    fn get_line(&self) -> usize {
        self.line
    }
    /// Returns the user-facing string form of the token.
    fn __str__(&self) -> PyResult<String> {
        Ok(if self.terminal.starts_with("__") {
            self.word.to_string()
        } else {
            format!("Token({}, \"{}\")", self.terminal, self.word)
        })
    }

    /// Returns the debug-style representation of the token.
    fn __repr__(&self) -> PyResult<String> {
        Ok(if self.terminal.starts_with("__") {
            self.word.to_string()
        } else {
            format!(
                "Token(Type: {}, Word: \"{}\", Start: {}, End: {}, Line: {})",
                self.terminal, self.word, self.start, self.end, self.line
            )
        })
    }
}

/// Tree
/// =====
/// Signature:
///     Tree(name: String, children: List[Tree | Token])
///
/// Arguments:
///     name:       String  | Name of tree (rule name).
///     children:   List    | List of Tree or Token.
#[pyclass]
pub struct Tree {
    name: String,
    children: Vec<Py<PyAny>>,
}

#[pymethods]
impl Tree {
    #[new]
    #[pyo3(signature = (name, children))]
    /// Creates a Python tree wrapper.
    pub fn new(name: String, children: Vec<Py<PyAny>>) -> Self {
        Self { name, children }
    }

    /// Returns child Python nodes.
    fn get_children(&self) -> &Vec<Py<PyAny>> {
        &self.children
    }

    /// Set child of tree
    /// Arguments:
    ///     children: List
    fn set_children(&mut self, children: Vec<Py<PyAny>>) {
        self.children = children;
    }

    /// Returns the tree node name.
    fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the start offset of the first descendant token.
    fn get_start(&self) -> usize {
        let first = self.children.first().unwrap();
        Python::attach(|py| {
            if first.bind(py).is_instance_of::<Tree>() {
                match first.call_method0(py, "get_start") {
                    Ok(val) => val.extract::<usize>(py).unwrap(),
                    Err(err) => {
                        panic!("Tree error: {}", err);
                    }
                }
            } else {
                let token = first.bind(py).cast::<Token>().unwrap().borrow();
                token.get_start()
            }
        })
    }

    /// Returns the end offset of the last descendant token.
    fn get_end(&self) -> usize {
        let last = self.children.last().unwrap();
        Python::attach(|py| {
            if last.bind(py).is_instance_of::<Tree>() {
                match last.call_method0(py, "get_end") {
                    Ok(val) => val.extract::<usize>(py).unwrap(),
                    Err(err) => {
                        panic!("Tree error: {}", err);
                    }
                }
            } else {
                let token = last.bind(py).cast::<Token>().unwrap().borrow();
                token.get_end()
            }
        })
    }

    /// Returns the user-facing string form of the tree.
    fn __str__(&self) -> PyResult<String> {
        let child = self
            .children
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        Ok(format!("Tree({}, [{}])", self.name, child))
    }

    /// Returns the debug-style representation of the tree.
    fn __repr__(&self) -> PyResult<String> {
        let child = self
            .children
            .iter()
            .map(|c| {
                let val = Python::attach(|python| c.call_method0(python, "__repr__"))
                    .expect("__repr__() failed");
                val.to_string()
            })
            .collect::<Vec<String>>()
            .join(", ");
        Ok(format!("Tree({}, [{}])", self.name, child))
    }
}

/// Converts a Rust AST node into the exported Python object graph.
fn convert_to_py(py: Python<'_>, ast: &AST) -> PyResult<Py<PyAny>> {
    match &ast {
        AST::Token(token) => {
            let py_token = Token {
                word: token.word().to_string(),
                start: token.get_start(),
                end: token.get_end(),
                line: token.get_line(),
                terminal: token.terminal.get_value(),
            };
            Ok(py_token.into_pyobject(py)?.into_any().unbind())
        }
        AST::Tree(name, children) => {
            let child = children
                .iter()
                .map(|child| convert_to_py(py, child).unwrap())
                .collect::<Vec<_>>();

            let tree = Tree {
                name: name.to_string(),
                children: child,
            };
            Ok(tree.into_pyobject(py)?.into_any().unbind())
        }
    }
}

/// Python wrapper around the Rust `Swiftlet` parser.
#[pyclass(module = "swiftlet._core")]
pub struct Swiftlet {
    inner: Mutex<RustSwiftlet>,
}

#[pymethods]
impl Swiftlet {
    #[new]
    #[pyo3(signature = (grammar, start="start", algorithm="earley", ambiguity="resolve", lexer_mode="basic", debug=false))]
    /// Constructs a parser from grammar text.
    fn new(
        grammar: &str,
        start: &str,
        algorithm: &str,
        ambiguity: &str,
        lexer_mode: &str,
        debug: bool,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Mutex::new(build_parser_from_grammar(
                grammar, start, algorithm, ambiguity, lexer_mode, debug,
            )?),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (file, start="start", algorithm="earley", ambiguity="resolve", lexer_mode="basic", debug=false))]
    /// Constructs a parser from a grammar file path.
    fn from_file(
        file: &str,
        start: &str,
        algorithm: &str,
        ambiguity: &str,
        lexer_mode: &str,
        debug: bool,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Mutex::new(build_parser_from_file(
                file, start, algorithm, ambiguity, lexer_mode, debug,
            )?),
        })
    }

    /// Parses input text and converts the resulting AST into Python objects.
    fn parse(&self, py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
        let parser = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("failed to acquire parser lock"))?;

        let parsed = catch_unwind(AssertUnwindSafe(|| parser.parse(text)))
            .map_err(|payload| PyRuntimeError::new_err(panic_payload_to_string(payload)))?;

        let ast = parsed.map_err(|err| PyValueError::new_err(err.to_string()))?;
        let py_ast = convert_to_py(py, &ast)?;
        Ok(py_ast)
    }
}

/// Initializes the Python extension module.
#[pymodule]
#[pyo3(name = "_swiftlet")]
fn core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Swiftlet>()?;
    module.add_class::<Tree>()?;
    module.add_class::<Token>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
