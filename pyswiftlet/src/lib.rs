use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use swiftlet::grammar::Algorithm as RustAlgorithm;
use swiftlet::lexer::AST;
use swiftlet::{Ambiguity as RustAmbiguity, Swiftlet as RustSwiftlet, ParserOption};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

fn parse_algorithm(value: &str) -> PyResult<RustAlgorithm> {
    match value.to_ascii_lowercase().as_str() {
        "earley" => Ok(RustAlgorithm::Earley),
        "clr" => Ok(RustAlgorithm::CLR),
        "lalr" => Ok(RustAlgorithm::LALR),
        _ => Err(PyValueError::new_err(format!(
            "invalid algorithm '{value}', expected 'earley', 'clr', or 'lalr'"
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
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    "swiftlet panicked".to_string()
}

fn build_parser_option(start: &str, algorithm: &str, ambiguity: &str, debug: bool) -> PyResult<Arc<ParserOption>> {
    Ok(Arc::new(ParserOption {
        start: start.to_string(),
        algorithm: parse_algorithm(algorithm)?,
        ambiguity: parse_ambiguity(ambiguity)?,
        debug,
    }))
}

fn build_parser_from_grammar(grammar: &str, start: &str, algorithm: &str, ambiguity: &str, debug: bool) -> PyResult<RustSwiftlet> {
    let parser_option = build_parser_option(start, algorithm, ambiguity, debug)?;
    catch_unwind(AssertUnwindSafe(|| {
        RustSwiftlet::from_string(grammar, parser_option)
    }))
    .map_err(|payload| PyRuntimeError::new_err(panic_payload_to_string(payload)))
}

fn build_parser_from_file(file: &str, start: &str, algorithm: &str, ambiguity: &str, debug: bool) -> PyResult<RustSwiftlet> {
    let parser_option = build_parser_option(start, algorithm, ambiguity, debug)?;
    catch_unwind(AssertUnwindSafe(|| {
        RustSwiftlet::from_file(file.to_string(), parser_option)
    }))
    .map_err(|payload| PyRuntimeError::new_err(panic_payload_to_string(payload)))
}

// fn ast_to_python(py: Python<'_>, ast: &AST) -> PyResult<Py<PyAst>> {
//     match ast {
//         AST::Token(token) => Py::new(
//             py,
//             PyAst {
//                 kind: "token",
//                 name: None,
//                 value: Some(token.word.clone()),
//                 terminal: Some(token.terminal.as_ref().as_str().to_string()),
//                 children: Vec::new(),
//                 text: OnceLock::from(format!("\"{}\"", token.word)),
//             },
//         ),
//         AST::Tree(name, children) => {
//             let py_children = children
//                 .iter()
//                 .map(|child| ast_to_python(py, child))
//                 .collect::<PyResult<Vec<_>>>()?;
//
//             Py::new(
//                 py,
//                 PyAst {
//                     kind: "tree",
//                     name: Some(name.clone()),
//                     value: None,
//                     terminal: None,
//                     children: py_children,
//                     text: OnceLock::new(),
//                 },
//             )
//         }
//     }
// }
//
// #[pyclass(module = "swiftlet._core", skip_from_py_object)]
// pub struct PyAst {
//     #[pyo3(get)]
//     kind: &'static str,
//     #[pyo3(get)]
//     name: Option<String>,
//     #[pyo3(get)]
//     value: Option<String>,
//     #[pyo3(get)]
//     terminal: Option<String>,
//     children: Vec<Py<PyAst>>,
//     text: OnceLock<String>,
// }
//
// #[pymethods]
// impl PyAst {
//     pub fn is_token(&self) -> bool {
//         self.kind == "token"
//     }
//
//     pub fn is_tree(&self) -> bool {
//         self.kind == "tree"
//     }
//
//     fn children(&self, py: Python<'_>) -> Vec<Py<PyAst>> {
//         self.children
//             .iter()
//             .map(|child| child.clone_ref(py))
//             .collect()
//     }
//
//     #[getter]
//     fn text(&self, py: Python<'_>) -> PyResult<String> {
//         if let Some(text) = self.text.get() {
//             return Ok(text.clone());
//         }
//
//         let text = self.compute_text(py)?;
//         let _ = self.text.set(text.clone());
//         Ok(text)
//     }
//
//     fn __repr__(&self) -> String {
//         Python::attach(|py| {
//             self.text
//                 .get_or_init(|| {
//                     self.compute_text(py)
//                         .unwrap_or_else(|_| "<failed to render AST>".to_string())
//                 })
//                 .clone()
//         })
//     }
// }
//
// impl PyAst {
//     fn compute_text(&self, py: Python<'_>) -> PyResult<String> {
//         if self.kind == "token" {
//             return Ok(format!(
//                 "\"{}\"",
//                 self.value.as_deref().unwrap_or_default()
//             ));
//         }
//
//         let children = self
//             .children
//             .iter()
//             .map(|child| child.bind(py).borrow().text(py))
//             .collect::<PyResult<Vec<_>>>()?;
//
//         Ok(format!(
//             "Tree(\"{}\", [{}])",
//             self.name.as_deref().unwrap_or_default(),
//             children.join(", ")
//         ))
//     }
// }

#[pyclass(module = "swiftlet._core", skip_from_py_object)]
pub struct PyAst {
    inner: AST
}

#[pymethods]
impl PyAst {
    fn is_token(&self) -> bool {
        match self.inner {
            AST::Token(_) => true,
            AST::Tree(_, _) => false,
        }
    }

    fn is_tree(&self) -> bool {
        !self.is_token()
    }

    fn get_text(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.inner.get_text())
    }

    fn pretty_print(&self) {
        self.inner.pretty_print();
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(self.inner.get_text())
    }
}

#[pyclass(module = "swiftlet._core")]
pub struct Swiftlet {
    inner: Mutex<RustSwiftlet>,
}

#[pymethods]
impl Swiftlet {
    #[new]
    #[pyo3(signature = (grammar, start="start", algorithm="earley", ambiguity="resolve", debug=false))]
    fn new(
        grammar: &str,
        start: &str,
        algorithm: &str,
        ambiguity: &str,
        debug: bool,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Mutex::new(build_parser_from_grammar(
                grammar, start, algorithm, ambiguity, debug,
            )?),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (file, start="start", algorithm="earley", ambiguity="resolve", debug=false))]
    fn from_file(
        file: &str,
        start: &str,
        algorithm: &str,
        ambiguity: &str,
        debug: bool,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Mutex::new(build_parser_from_file(
                file, start, algorithm, ambiguity, debug,
            )?),
        })
    }

    fn parse(&self, py: Python<'_>, text: &str) -> PyResult<Py<PyAst>> {
        let parser = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("failed to acquire parser lock"))?;

        let parsed = catch_unwind(AssertUnwindSafe(|| parser.parse(text)))
            .map_err(|payload| PyRuntimeError::new_err(panic_payload_to_string(payload)))?;

        let ast = parsed.map_err(|err| PyValueError::new_err(err.to_string()))?;
        // let ast_to_py = ast_to_python(py, &ast);

        // ast_to_py.map_err(|msg| PyRuntimeError::new_err(msg.to_string()))
        Ok(Py::new(py, PyAst {inner: ast})?)
    }
}

#[pymodule]
#[pyo3(name = "_core")]
fn core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Swiftlet>()?;
    // module.add_class::<PyAst>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
